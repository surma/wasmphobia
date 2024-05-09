use std::{
    collections::HashMap,
    io::{Read, Write},
    path::PathBuf,
};

use anyhow::Context;

use clap::Parser;
use object::{Object, ObjectSection};

#[derive(Clone, Debug, Parser)]
#[command(version)]
struct Args {
    #[arg(short, long)]
    input: Option<PathBuf>,
    #[arg(short, long)]
    output: Option<PathBuf>,

    #[arg(long)]
    /// Title for the flame graph (default: input file name)
    title: Option<String>,
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

struct Segment {
    name: String,
    start: u64,
    end: u64,
    mapped: u64,
}

impl Segment {
    fn size(&self) -> u64 {
        self.end - self.start
    }
}

fn main() -> anyhow::Result<()> {
    let stdinout_marker: PathBuf = PathBuf::from("-");

    let args = Args::parse();
    let input_data = match &args.input {
        Some(path) if path != &stdinout_marker => std::fs::read(path)?,
        _ => read_stdin()?,
    };
    let module_size = input_data.len();

    let wasm_file = object::wasm::WasmFile::parse(input_data.as_slice())?;

    let mut segments: Vec<_> = wasm_file
        .sections()
        .filter_map(|s| {
            let (start, end) = s.file_range()?;
            Some(Segment {
                name: s.name().ok()?.to_string(),
                start,
                end,
                mapped: 0,
            })
        })
        .collect();

    let context = addr2line::Context::new(&wasm_file)?;

    let mut contributors = HashMap::new();
    for (map_start, size, loc) in context.find_location_range(0, module_size.try_into().unwrap())? {
        let map_end = map_start + size;
        let section_name = if let Some(section) = segments
            .iter_mut()
            .find(|s| s.start <= map_start && s.end > map_end)
        {
            section.mapped += size;
            section.name.as_str()
        } else {
            "<unknown section>"
        };
        let file = loc.file.unwrap_or("<unknown file>");
        let line = loc
            .line
            .map(|s| format!("{s}"))
            .unwrap_or_else(|| "<unknown>".to_string());

        let key = format!(
            "@section: {section_name};{};@line: {line}_",
            file.trim_start_matches('/').replace('/', ";")
        );
        *contributors.entry(key).or_insert(0) += size;
    }

    for segment in segments {
        let key = format!("@section: {};<no mapping info>", segment.name);
        *contributors.entry(key).or_insert(0) += segment.size() - segment.mapped;
    }

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
