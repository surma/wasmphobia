use std::{
    collections::HashMap,
    io::{Read, Write},
    ops::{Range, Sub},
    path::PathBuf,
};

use anyhow::{anyhow, Context};

use clap::Parser;

mod dwarf;
use dwarf::DwarfAnalysisOpts;
use gimli::{EndianSlice, LittleEndian};

#[derive(Clone, Debug, Parser)]
#[command(version)]
struct Args {
    #[arg(short, long)]
    input: Option<PathBuf>,
    #[arg(short, long)]
    output: Option<PathBuf>,

    #[arg(long, default_value_t = false)]
    /// Group by DWARF compilation units
    compilation_units: bool,

    #[arg(long, default_value_t = false)]
    /// Split file paths at each folder in the flame graph
    split_paths: bool,

    #[arg(long)]
    /// Title for the flame graph (default: input file name)
    title: Option<String>,
}

impl From<Args> for DwarfAnalysisOpts {
    fn from(val: Args) -> Self {
        DwarfAnalysisOpts {
            prefix: None,
            compilation_units: val.compilation_units,
            split_paths: val.split_paths,
        }
    }
}

impl From<Args> for inferno::flamegraph::Options<'static> {
    fn from(value: Args) -> Self {
        let mut options = inferno::flamegraph::Options::default();
        options.title = value
            .title
            .or_else(|| Some(value.input.as_ref()?.file_name()?.to_str()?.to_string()))
            .unwrap_or("<Unknown wasm file>".to_string());
        options.subtitle =
            Some("Contribution to WebAssembly module size per DWARF compilation unit".to_string());
        options.count_name = "KB".to_string();
        options.factor = 1.0 / 1000.0;
        options.name_type = "".to_string();
        options
    }
}

fn main() -> anyhow::Result<()> {
    let stdinout_marker: PathBuf = PathBuf::from("-");

    let args = Args::parse();
    let input_data = match &args.input {
        Some(path) if path != &stdinout_marker => std::fs::read(path)?,
        _ => read_stdin()?,
    };

    let module = walrus::Module::from_buffer(&input_data).context("Parsing WebAssembly")?;
    let dwarf = module.debug.dwarf;
    let dwarf = dwarf.borrow(|v| EndianSlice::new(v.as_slice(), LittleEndian));

    const WASM_SECTION_PREFIX: &str = "@wasm_binary_module;@section: ";
    let wasm_code_section = format!("{WASM_SECTION_PREFIX}code");

    let mut contributors = dwarf::analyze_dwarf(
        dwarf,
        &DwarfAnalysisOpts {
            prefix: Some(wasm_code_section.clone()),
            ..args.clone().into()
        },
    )
    .context("Analyzing DWARF data")?;

    let mut wasm_section_sizes = section_sizes(&input_data).context("Analyzing Wasm sections")?;

    let mapped_wasm_code_size: u64 = contributors.values().sum();
    let total_code_size = wasm_section_sizes
        .remove("code")
        .ok_or_else(|| anyhow!("Wasm module without a code section"))?;
    if let Some(unmapped_wasm_code_size) = total_code_size.checked_sub(mapped_wasm_code_size) {
        contributors.insert(
            format!("{wasm_code_section};<unmapped>"),
            unmapped_wasm_code_size,
        );
    } else {
        eprintln!(
            "[Warning] Mapped code regions add up to more bytes than the Wasm's code section"
        );
    }

    contributors.extend(
        wasm_section_sizes
            .into_iter()
            .map(|(key, val)| (format!("{WASM_SECTION_PREFIX}{key}"), val)),
    );

    let output: Box<dyn Write> = match &args.output {
        Some(path) if path != &stdinout_marker => Box::new(std::fs::File::create(path)?),
        _ => Box::new(std::io::stdout()),
    };

    write_flamegraph(contributors, args.into(), output).context("Rendering flame graph")?;

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

fn read_stdin() -> std::io::Result<Vec<u8>> {
    let mut buf = vec![];
    std::io::stdin().read_to_end(&mut buf)?;
    Ok(buf)
}

fn range_size<T: Sub<Output = T>>(r: Range<T>) -> T {
    r.end - r.start
}

fn section_sizes(mut module: &[u8]) -> anyhow::Result<HashMap<String, u64>> {
    use wasmparser::{Chunk, Parser, Payload};
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
        sections.insert(name, size.try_into().unwrap());
    }

    Ok(sections)
}
