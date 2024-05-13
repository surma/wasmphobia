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
