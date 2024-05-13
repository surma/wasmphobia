use addr2line::fallible_iterator::FallibleIterator;
use anyhow::Context;

use super::{dwarf::functions_for_address, BundleAnalysis, BundleAnalysisConfig, BundleFormat};

pub struct MachoBundle;

const MARKER1: &[u8] = &[0xce, 0xfa, 0xed, 0xfe];
const MARKER2: &[u8] = &[0xcf, 0xfa, 0xed, 0xfe];
impl BundleFormat for MachoBundle {
    fn can_handle(input_data: &[u8]) -> bool {
        let header = &input_data[0..MARKER1.len()];
        header == MARKER1 || header == MARKER2
    }

    fn analyze(config: &BundleAnalysisConfig, input_data: &[u8]) -> anyhow::Result<BundleAnalysis> {
        let _module_size = input_data.len();
        let obj = object::read::File::parse(input_data)?;
        println!("{:?}", obj.format());
        let context = addr2line::Context::new(&obj).context("Constructing address mapping")?;
        let mut contributors = BundleAnalysis::default();
        let locations: Vec<_> =
            FallibleIterator::collect(context.find_location_range(0, u64::MAX)?)?;
        dbg!(locations.len());
        for (map_start, size, loc) in locations.into_iter().rev() {
            let file = loc.file.unwrap_or("<unknown file>");

            let mut key = file.trim_start_matches('/').replace('/', ";").to_string();

            if !config.files_only {
                let funcs = functions_for_address(config, &context, map_start)?;
                key = format!("{key};{}", funcs.join(";"));
            }

            *contributors.entry(key).or_insert(0) += size;
        }
        Ok(contributors)
    }
}
