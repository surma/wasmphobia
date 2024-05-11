use std::{
    collections::HashMap,
    io::{Read, Write},
    ops::Range,
    path::PathBuf,
};

use addr2line::{
    fallible_iterator::FallibleIterator,
    gimli::{read::Dwarf, EndianSlice, LittleEndian},
};
use anyhow::Context;

use clap::Parser;

#[derive(Clone, Debug, Parser)]
#[command(version)]
struct Args {
    #[arg(short, long)]
    input: Option<PathBuf>,
    #[arg(short, long)]
    output: Option<PathBuf>,

    #[arg(long)]
    /// Only break down to files, not functions.
    files_only: bool,

    #[arg(long)]
    /// Show raw object symbol names for functions, rather than demangling them.
    raw_symbols: bool,

    #[arg(long)]
    /// Title for the flame graph (default: input file name).
    title: Option<String>,

    #[arg(long)]
    /// Show DWARF debug sections in the breakdown.
    show_debug_sections: bool,

    #[arg(long, default_value_t = 32)]
    /// Minimum size of a mapped region in bytes to be shown in the flamegraph. (WARNING: Small values can make the flamegraph very big and slow.)
    size_threshold: usize,
}

impl From<Args> for inferno::flamegraph::Options<'static> {
    fn from(value: Args) -> Self {
        let mut options = inferno::flamegraph::Options::default();
        options.title = value
            .title
            .or_else(|| Some(value.input.as_ref()?.file_name()?.to_str()?.to_string()))
            .unwrap_or("<Unknown file>".to_string());
        options.subtitle = Some("File size breakdown".to_string());
        options.count_name = "KB".to_string();
        options.factor = 1.0 / 1000.0;
        options.min_width = value.size_threshold as f64 / 1000.0;
        options.frame_height = 24;
        options.name_type = "".to_string();
        options
    }
}

struct Section {
    name: String,
    start: u64,
    end: u64,
    mapped: u64,
}

impl Section {
    fn size(&self) -> u64 {
        self.end - self.start
    }
}

fn main() -> anyhow::Result<()> {
    let stdinout_marker: PathBuf = PathBuf::from("-");

    let args = Args::parse();
    let input_data = match &args.input {
        Some(path) if path != &stdinout_marker => std::fs::read(path).context("Reading input")?,
        _ => read_stdin()?,
    };

    let contributors = if input_data[0] == b'{' {
        analyze_sourcemaps(&args, input_data).context("Analyzing sourcemaps")?
    } else {
        analyze_wasm(&args, input_data).context("Analyzing wasm")?
    };

    let output: Box<dyn Write> = match &args.output {
        Some(path) if path != &stdinout_marker => Box::new(std::fs::File::create(path)?),
        _ => Box::new(std::io::stdout()),
    };

    write_flamegraph(contributors, args.into(), output).context("Rendering flame graph")?;

    Ok(())
}

fn analyze_sourcemaps(_args: &Args, input_data: Vec<u8>) -> anyhow::Result<HashMap<String, u64>> {
    use sourcemap::SourceMap;
    let sm = SourceMap::from_slice(&input_data)?;
    let mut contributors = HashMap::new();
    let mut prev_line = 0;
    let mut prev_col = 0;
    for (line, col, idx) in sm.index_iter() {
        let size = if prev_line == line {
            col - prev_col
        } else {
            col
        };
        (prev_line, prev_col) = (line, col);

        let token = sm.get_token(idx).expect("Index given by index iterator");
        let source_file = token
            .get_source()
            .unwrap_or("<unknown file>")
            .split('/')
            .collect::<Vec<_>>()
            .join(";");
        *contributors.entry(source_file).or_insert(0) += u64::from(size);
    }
    Ok(contributors)
}

fn analyze_wasm(args: &Args, input_data: Vec<u8>) -> anyhow::Result<HashMap<String, u64>> {
    let module_size = input_data.len();
    let (dwarf, mut sections) = parse_wasm(&input_data).context("Parsing Wasm")?;
    if !args.show_debug_sections {
        sections.retain(|sect| !sect.name.starts_with(".debug"));
    }
    let context = addr2line::Context::from_dwarf(dwarf).context("Constructing address mapping")?;
    let mut contributors = HashMap::new();
    let locations: Vec<_> = FallibleIterator::collect(
        context.find_location_range(0, module_size.try_into().unwrap())?,
    )?;
    for (map_start, size, loc) in locations.into_iter().rev() {
        let map_end = map_start + size;
        let section_name = if let Some(section) = sections
            .iter_mut()
            .find(|s| s.start <= map_start && s.end > map_end)
        {
            section.mapped += size;
            section.name.as_str()
        } else {
            "<unknown section>"
        };
        let file = loc.file.unwrap_or("<unknown file>");

        let mut key = format!(
            "@section: {section_name};{}",
            file.trim_start_matches('/').replace('/', ";")
        );

        if !args.files_only {
            let funcs = functions_for_address(args, &context, map_start)?;
            key = format!("{key};{}", funcs.join(";"));
        }

        *contributors.entry(key).or_insert(0) += size;
    }
    for segment in sections {
        let key = format!("@section: {};<no mapping info>", segment.name);
        *contributors.entry(key).or_insert(0) += segment.size() - segment.mapped;
    }
    Ok(contributors)
}

fn parse_wasm<'a>(
    buf: &'a [u8],
) -> anyhow::Result<(Dwarf<EndianSlice<'a, LittleEndian>>, Vec<Section>)> {
    use wasmparser::{Parser, Payload, Payload::*};

    static EMPTY_SECTION: &[u8] = &[];

    let parser = Parser::new(0);
    let mut dwarf_sections: HashMap<&'a str, &'a [u8]> = HashMap::new();
    let mut sections = vec![];
    for payload in parser.parse_all(buf) {
        let (name, range) = match payload? {
            CustomSection(section) => {
                let name = section.name();
                if name.starts_with(".debug") {
                    dwarf_sections.insert(name, section.data());
                }
                let start: usize = section.data_offset();
                let end = start + section.data().len();
                (name.to_string(), Range { start, end })
            }

            TypeSection(s) => ("type".to_string(), s.range()),
            ImportSection(s) => ("import".to_string(), s.range()),
            FunctionSection(s) => ("function".to_string(), s.range()),
            TableSection(s) => ("table".to_string(), s.range()),
            MemorySection(s) => ("memory".to_string(), s.range()),
            TagSection(s) => ("tag".to_string(), s.range()),
            GlobalSection(s) => ("global".to_string(), s.range()),
            ExportSection(s) => ("export".to_string(), s.range()),
            ElementSection(s) => ("element".to_string(), s.range()),
            DataSection(s) => ("data".to_string(), s.range()),
            CodeSectionStart { range, .. } => ("code".to_string(), range),
            InstanceSection(s) => ("instance".to_string(), s.range()),
            CoreTypeSection(s) => ("core type".to_string(), s.range()),
            // FIXME: Is recursion needed here?
            ComponentSection {
                unchecked_range, ..
            } => ("component".to_string(), unchecked_range),
            ComponentInstanceSection(s) => ("component instance".to_string(), s.range()),
            ComponentAliasSection(s) => ("component alias".to_string(), s.range()),
            ComponentTypeSection(s) => ("component type".to_string(), s.range()),
            ComponentCanonicalSection(s) => ("component canonical".to_string(), s.range()),
            ComponentImportSection(s) => ("component import".to_string(), s.range()),
            ComponentExportSection(s) => ("component export".to_string(), s.range()),

            Payload::End(_) => break,
            _ => continue,
        };
        sections.push(Section {
            name,
            start: range.start.try_into().unwrap(),
            end: range.end.try_into().unwrap(),
            mapped: 0,
        });
    }

    let dwarf = Dwarf::load(|section_id| -> anyhow::Result<_> {
        let data = *dwarf_sections
            .get(section_id.name())
            .unwrap_or(&EMPTY_SECTION);
        Ok(EndianSlice::new(data, LittleEndian))
    })?;

    Ok((dwarf, sections))
}

fn functions_for_address<R: addr2line::gimli::Reader>(
    args: &Args,
    context: &addr2line::Context<R>,
    map_start: u64,
) -> anyhow::Result<Vec<String>> {
    let funcs: Vec<_> = context
        .find_frames(map_start)
        .skip_all_loads()?
        .filter_map(|frame| {
            let mut name = if let Some(function) = frame.function {
                function.name.to_string_lossy()?.to_string()
            } else {
                "<Unknown>".to_string()
            };
            if !args.raw_symbols {
                if let Ok(demangled) = rustc_demangle::try_demangle(&name) {
                    name = demangled.to_string();
                }
                if let Ok(demangled) = cpp_demangle::Symbol::new(name.clone()) {
                    name = demangled.to_string();
                }
            }
            Ok(Some(format!("@function: {name}")))
        })
        .collect()?;
    Ok(funcs)
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
