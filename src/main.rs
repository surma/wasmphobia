use std::{
    collections::HashMap,
    io::{Read, Write},
    ops::{Range, Sub},
    path::PathBuf,
};

use anyhow::anyhow;
use fallible_iterator::FallibleIterator;
use gimli::{read::AttributeValue, DebuggingInformationEntry, EndianSlice, LittleEndian, Unit};

#[cfg(feature = "cli-args")]
use clap::Parser;

#[derive(Default, Debug)]
#[cfg_attr(feature = "cli-args", derive(Parser))]
#[cfg_attr(feature = "cli-args", command(version))]
struct Args {
    #[cfg_attr(feature = "cli-args", arg(short, long))]
    input: Option<PathBuf>,
    #[cfg_attr(feature = "cli-args", arg(short, long))]
    output: Option<PathBuf>,
}

#[cfg(not(feature = "cli-args"))]
impl Args {
    fn parse() -> Self {
        Default::default()
    }
}

fn main() -> anyhow::Result<()> {
    let stdinout_marker: PathBuf = PathBuf::from("-");

    let args = Args::parse();
    let input_data = match &args.input {
        Some(path) if path != &stdinout_marker => std::fs::read(path)?,
        _ => read_stdin()?,
    };

    let module = walrus::Module::from_buffer(&input_data)?;
    let dwarf = module.debug.dwarf;
    let dwarf = dwarf.borrow(|v| EndianSlice::new(v.as_slice(), LittleEndian));

    const WASM_SECTION_PREFIX: &str = "@wasm_binary_module;@section: ";
    let wasm_code_section = format!("{WASM_SECTION_PREFIX}code");
    let mut contributors = accumulate_contributors(Some(&format!("{wasm_code_section};")), dwarf)?;

    let mut wasm_section_sizes = section_sizes(Some(WASM_SECTION_PREFIX), &input_data)?;
    let mapped_wasm_code_size: u64 = contributors.values().sum();
    let total_code_size = wasm_section_sizes
        .remove(&wasm_code_section)
        .ok_or_else(|| anyhow!("Wasm module without a code section"))?;
    let unmapped_wasm_code_size = total_code_size - mapped_wasm_code_size;
    contributors.extend(wasm_section_sizes);
    contributors.insert(
        format!("{wasm_code_section};<unmapped>"),
        unmapped_wasm_code_size,
    );

    let output: Box<dyn Write> = match &args.output {
        Some(path) if path != &stdinout_marker => Box::new(std::fs::File::create(path)?),
        _ => Box::new(std::io::stdout()),
    };

    let options = create_flamegraph_config(args);

    write_flamegraph(contributors, options, output)?;

    Ok(())
}

fn write_flamegraph(
    contributors: HashMap<String, u64>,
    mut options: inferno::flamegraph::Options<'_>,
    mut output: Box<dyn Write>,
) -> anyhow::Result<()> {
    let inferno_lines: Vec<_> = contributors
        .into_iter()
        .map(|(key, size)| format!("{} {}", key, size))
        .collect();
    inferno::flamegraph::from_lines(
        &mut options,
        inferno_lines.iter().map(|v| v.as_str()),
        &mut output,
    )?;
    Ok(())
}

fn create_flamegraph_config(args: Args) -> inferno::flamegraph::Options<'static> {
    let mut options = inferno::flamegraph::Options::default();
    options.title = args
        .input
        .as_ref()
        .and_then(|s| s.file_name())
        .and_then(|s| s.to_str())
        .unwrap_or("<Unknown wasm file>")
        .to_string();
    options.subtitle =
        Some("Contribution to WebAssembly module size per DWARF compilation unit".to_string());
    options.count_name = "KB".to_string();
    options.factor = 1.0 / 1000.0;
    options.name_type = "".to_string();
    options
}

fn read_stdin() -> std::io::Result<Vec<u8>> {
    let mut buf = vec![];
    std::io::stdin().read_to_end(&mut buf)?;
    Ok(buf)
}

fn range_size<T: Sub<Output = T>>(r: Range<T>) -> T {
    r.end - r.start
}

fn section_sizes(prefix: Option<&str>, mut module: &[u8]) -> anyhow::Result<HashMap<String, u64>> {
    use wasmparser::{Chunk, Parser, Payload};
    let prefix = prefix.unwrap_or("");
    let mut sections: HashMap<String, u64> = HashMap::new();
    let mut cur = Parser::new(0);

    loop {
        let Chunk::Parsed { payload, consumed } = cur.parse(module, true)? else {
            anyhow::bail!("Incomplete wasm file")
        };
        module = &module[consumed..];

        let (name, size) = match payload {
            // Sections for WebAssembly modules
            Payload::TypeSection(s) => ("type".to_string(), range_size(s.range())),
            Payload::DataSection(s) => ("data".to_string(), range_size(s.range())),
            Payload::CustomSection(s) => (format!("custom;{}", s.name()), range_size(s.range())),
            Payload::FunctionSection(s) => ("function".to_string(), range_size(s.range())),
            Payload::ImportSection(s) => ("import".to_string(), range_size(s.range())),
            Payload::TableSection(s) => ("table".to_string(), range_size(s.range())),
            Payload::MemorySection(s) => ("memory".to_string(), range_size(s.range())),
            Payload::ExportSection(s) => ("export".to_string(), range_size(s.range())),
            Payload::GlobalSection(s) => ("global".to_string(), range_size(s.range())),
            Payload::ElementSection(s) => ("element".to_string(), range_size(s.range())),
            Payload::UnknownSection { range, .. } => ("<unknown>".to_string(), range_size(range)),
            Payload::CodeSectionStart { range, .. } => ("code".to_string(), range_size(range)),
            Payload::End(_) => break,
            _ => continue,
        };
        sections.insert(format!("{prefix}{name}"), size.try_into().unwrap());
    }

    Ok(sections)
}

macro_rules! unwrap_or_continue {
    ($v:expr) => {
        match $v {
            Some(v) => v,
            _ => continue,
        }
    };
}

fn unpack_size<R: gimli::Reader>(low: &AttributeValue<R>, high: &AttributeValue<R>) -> Option<u64> {
    let AttributeValue::Addr(low) = *low else {
        return None;
    };
    match high {
        AttributeValue::Addr(v) => Some(*v - low),
        AttributeValue::Udata(v) => Some(*v),
        _ => None,
    }
}

fn accumulate_contributors(
    prefix: Option<&str>,
    dwarf: gimli::Dwarf<EndianSlice<'_, LittleEndian>>,
) -> Result<HashMap<String, u64>, anyhow::Error> {
    let prefix = prefix.unwrap_or("");
    let mut contributors: HashMap<String, u64> = HashMap::new();
    let mut iter = dwarf.units();
    while let Some(header) = iter.next()? {
        let unit = dwarf.unit(header)?;
        let unit_name = unit
            .name
            .and_then(|s| s.to_string().ok())
            .unwrap_or("<unknown compilation unit>")
            .trim_start_matches('/');
        let mut entries = unit.entries();
        while let Some((_, entry)) = entries.next_dfs()? {
            match entry.tag() {
                gimli::DW_TAG_subprogram | gimli::DW_TAG_inlined_subroutine => {}
                _ => continue,
            };

            let file = entry.attr_value(gimli::DW_AT_decl_file)?;
            let func_name = unwrap_or_continue!(entry.attr_value(gimli::DW_AT_name)?);

            let (dir, file) =
                unpack_file(file, &unit, &dwarf).unwrap_or(("<unknown dir>", "<unknown file>"));
            let dir = dir.trim_start_matches('/');
            let func_name =
                unwrap_or_continue!(func_name.string_value(&dwarf.debug_str)).to_string()?;
            let size = unwrap_or_continue!(entry_mapped_size(entry, &unit, &dwarf)?);
            let key = format!(
                "{prefix}@compilation_unit: {unit_name};@source_file: {dir}/{file};{func_name}"
            );
            *contributors.entry(key).or_insert(0) += size;
        }
    }
    Ok(contributors)
}

macro_rules! unwrap_or_ok_none {
    ($v:expr) => {
        match $v {
            Some(v) => v,
            _ => return Ok(None),
        }
    };
}

fn entry_mapped_size<R: gimli::Reader>(
    entry: &DebuggingInformationEntry<'_, '_, R>,
    unit: &Unit<R>,
    dwarf: &gimli::Dwarf<R>,
) -> anyhow::Result<Option<u64>> {
    let low_pc = entry.attr_value(gimli::DW_AT_low_pc)?;
    let size = if let Some(low_pc) = low_pc {
        let high_pc = unwrap_or_ok_none!(entry.attr_value(gimli::DW_AT_high_pc)?);
        unwrap_or_ok_none!(unpack_size(&low_pc, &high_pc))
    } else {
        let ranges = unwrap_or_ok_none!(entry.attr_value(gimli::DW_AT_ranges)?);
        // ranges.offset_value()
        let AttributeValue::RangeListsRef(list_ref) = ranges else {
            return Ok(None);
        };
        let range_list_offset = dwarf.ranges_offset_from_raw(unit, list_ref);
        let ranges = dwarf.ranges(unit, range_list_offset)?;
        ranges
            .map(|range| Ok(range.end - range.begin))
            .fold(0, |acc, d| Ok(acc + d))?
    };
    Ok(Some(size))
}

fn unpack_file<'i>(
    file: Option<AttributeValue<EndianSlice<'i, LittleEndian>, usize>>,
    unit: &gimli::Unit<EndianSlice<'i, LittleEndian>, usize>,
    dwarf: &gimli::Dwarf<EndianSlice<'i, LittleEndian>>,
) -> Option<(&'i str, &'i str)> {
    let AttributeValue::FileIndex(file_index) = file? else {
        return None;
    };
    let header = unit.line_program.as_ref()?.header();
    let file = header.file(file_index)?;
    let dir = file
        .directory(header)?
        .string_value(&dwarf.debug_str)?
        .to_string()
        .ok()?;
    let name = file
        .path_name()
        .string_value(&dwarf.debug_str)?
        .to_string()
        .ok()?;
    Some((dir, name))
}
