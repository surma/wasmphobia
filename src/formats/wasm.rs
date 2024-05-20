use std::{collections::HashMap, ops::Range};

use log::warn;

use addr2line::{
    fallible_iterator::FallibleIterator,
    gimli::{read::Dwarf, EndianSlice, LittleEndian},
};
use anyhow::Context;

use super::{dwarf::functions_for_address, BundleAnalysis, BundleAnalysisConfig, BundleFormat};

pub struct Section {
    name: String,
    start: u64,
    end: u64,
    mapped: u64,
}

impl Section {
    pub fn size(&self) -> u64 {
        self.end - self.start
    }
}

pub struct WasmBundle;

const WASM_MAGIC_NUMBER: &[u8] = &[0x00, 0x61, 0x73, 0x6d];
impl BundleFormat for WasmBundle {
    fn name() -> String {
        "Wasm".into()
    }
    fn can_handle(input_data: &[u8]) -> bool {
        &input_data[0..WASM_MAGIC_NUMBER.len()] == WASM_MAGIC_NUMBER
    }

    fn analyze(config: &BundleAnalysisConfig, input_data: &[u8]) -> anyhow::Result<BundleAnalysis> {
        let module_size = input_data.len();
        let (dwarf, mut sections) = parse_wasm(input_data).context("Parsing Wasm")?;
        if !config.retain_debug_sections {
            sections.retain(|sect| !sect.name.starts_with(".debug"));
        }
        let context =
            addr2line::Context::from_dwarf(dwarf).context("Constructing address mapping")?;
        let mut contributors = BundleAnalysis::default();
        let locations: Vec<_> = FallibleIterator::collect(
            context
                .find_location_range(0, module_size.try_into().unwrap())
                .context("Find locations")?,
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

            if !config.files_only {
                let funcs = functions_for_address(config, &context, map_start)
                    .inspect_err(|err| warn!("Could not extract function names for region: {err}"))
                    .unwrap_or_default();
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
}

pub fn parse_wasm<'a>(
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
