// TODO: Implement DWARF standalone file format support

use addr2line::fallible_iterator::FallibleIterator;

use super::BundleAnalysisConfig;

pub fn functions_for_address<R: addr2line::gimli::Reader>(
    config: &BundleAnalysisConfig,
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
            if !config.raw_symbols {
                if let Ok(demangled) = demangle_rust(&name) {
                    name = demangled;
                } else if let Ok(demangled) = demangle_cpp(&name) {
                    name = demangled;
                } else {
                    name = format!("{name} (demangling failed)");
                }
            }
            Ok(Some(format!("@function: {name}")))
        })
        .collect()?;
    Ok(funcs)
}

fn demangle_cpp(name: impl AsRef<str>) -> anyhow::Result<String> {
    let symbol = cpp_demangle::Symbol::new(name.as_ref())?;
    Ok(symbol.demangle(&Default::default())?)
}

fn demangle_rust(name: impl AsRef<str>) -> anyhow::Result<String> {
    Ok(rustc_demangle::try_demangle(name.as_ref())
        .map_err(|err| anyhow::anyhow!("{err:?}"))?
        .to_string())
}
