use std::{
    collections::HashMap,
    io::{Read, Write},
    ops::{Range, Sub},
    path::PathBuf,
};

use anyhow::anyhow;
use gimli::{EndianSlice, LittleEndian};

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

    const WASM_CODE_SECTION: &str = "@wasm_binary/sections/code";
    let mut contributors =
        accumulate_contributors(Some((WASM_CODE_SECTION.to_string() + "/").as_str()), dwarf)?;
    let mut wasm_section_sizes = section_sizes(Some("@wasm_binary/sections/"), &input_data)?;
    let mapped_wasm_code_size: u64 = contributors.values().sum();
    let total_code_size = wasm_section_sizes
        .remove(WASM_CODE_SECTION)
        .ok_or_else(|| anyhow!("Wasm module without a code section"))?;
    let unmapped_wasm_code_size = total_code_size - mapped_wasm_code_size;
    contributors.extend(wasm_section_sizes);
    contributors.insert(
        format!("{WASM_CODE_SECTION}/<unmapped>"),
        unmapped_wasm_code_size,
    );

    let mut output: Box<dyn Write> = match &args.output {
        Some(path) if path != &stdinout_marker => Box::new(std::fs::File::create(path)?),
        _ => Box::new(std::io::stdout()),
    };

    let inferno_lines: Vec<_> = contributors
        .into_iter()
        .map(|(key, size)| {
            let inferno_key = key.replace(['/', '\\'], ";");
            format!("{} {}", inferno_key, size)
        })
        .collect();

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
    inferno::flamegraph::from_lines(
        &mut options,
        inferno_lines.iter().map(|v| v.as_str()),
        &mut output,
    )?;

    Ok(())
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
            Payload::TypeSection(s) => ("types".to_string(), range_size(s.range())),
            Payload::DataSection(s) => ("data".to_string(), range_size(s.range())),
            Payload::CustomSection(s) => (format!("custom/{}", s.name()), range_size(s.range())),
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
        sections.insert(prefix.to_string() + name.as_str(), size.try_into().unwrap());
    }

    Ok(sections)
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
        let name = prefix.to_string()
            + unit
                .name
                .map(|s| std::str::from_utf8(s.slice()).unwrap_or("<Invalid utf8>"))
                .unwrap_or("<Unknown>");

        let mut size = 0;
        let mut ranges = dwarf.unit_ranges(&unit)?;
        while let Some(range) = ranges.next()? {
            size += range.end - range.begin;
        }

        *contributors.entry(name).or_insert(0) += size;
    }
    Ok(contributors)
}
