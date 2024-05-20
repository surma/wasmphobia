use std::{
    collections::HashMap,
    io::{Read, Write},
    path::PathBuf,
};

use anyhow::Context;

use clap::Parser;
use formats::{
    analyze_bundle,
    sourcemaps::{EmbeddedSourceMapBundle, RawSourceMapBundle},
    wasm::WasmBundle,
};
use inferno::flamegraph::TextTruncateDirection;
use log::info;

mod formats;

#[derive(Clone, Debug, Parser)]
#[command(version)]
pub struct Args {
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
        options.text_truncate_direction = TextTruncateDirection::Right;
        options
    }
}

fn main() -> anyhow::Result<()> {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Debug)
        .parse_default_env()
        .init();
    info!("Starting...");
    let stdinout_marker: PathBuf = PathBuf::from("-");

    let args = Args::parse();
    let input_data = match &args.input {
        Some(path) if path != &stdinout_marker => std::fs::read(path).context("Reading input")?,
        _ => read_stdin()?,
    };

    let bundle_analysis = analyze_bundle_with_formats!(
        &args.clone().into(),
        &input_data,
        WasmBundle,
        RawSourceMapBundle,
        EmbeddedSourceMapBundle,
    )
    .context("Analyzing bundle")?;

    let output: Box<dyn Write> = match &args.output {
        Some(path) if path != &stdinout_marker => Box::new(std::fs::File::create(path)?),
        _ => Box::new(std::io::stdout()),
    };

    write_flamegraph(&bundle_analysis, args.into(), output).context("Rendering flame graph")?;

    Ok(())
}

fn write_flamegraph(
    contributors: &HashMap<String, u64>,
    mut options: inferno::flamegraph::Options<'_>,
    mut output: Box<dyn Write>,
) -> anyhow::Result<()> {
    let inferno_lines: Vec<_> = contributors
        .iter()
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
