use anyhow::{anyhow, Context};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};

use crate::formats::BundleAnalysis;

use super::{BundleAnalysisConfig, BundleFormat};

pub struct RawSourceMapBundle;
impl BundleFormat for RawSourceMapBundle {
    fn name() -> String {
        "SourceMap".into()
    }

    fn can_handle(input_data: &[u8]) -> bool {
        const MARKER: &[u8] = b"\"mappings\"";
        if input_data[0] != b'{' {
            return false;
        }
        input_data
            .windows(MARKER.len())
            .any(|chunk| chunk == MARKER)
    }

    fn analyze(
        _config: &BundleAnalysisConfig,
        input_data: &[u8],
    ) -> anyhow::Result<BundleAnalysis> {
        use sourcemap::SourceMap;
        let sm = SourceMap::from_slice(input_data)?;
        let mut contributors = BundleAnalysis::default();
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
}

const REF_MARKER: &[u8] = b"sourceMappingURL=data:";
pub struct EmbeddedSourceMapBundle;
impl BundleFormat for EmbeddedSourceMapBundle {
    fn name() -> String {
        "Embedded SourceMap".into()
    }

    fn can_handle(input_data: &[u8]) -> bool {
        const BASE64_MARKER: &[u8] = b"base64,ey";
        input_data
            .windows(REF_MARKER.len())
            .any(|chunk| chunk == REF_MARKER)
            && input_data
                .windows(BASE64_MARKER.len())
                .any(|chunk| chunk == BASE64_MARKER)
    }
    fn analyze(config: &BundleAnalysisConfig, data: &[u8]) -> anyhow::Result<BundleAnalysis> {
        let sourcemap = unembed_sourcemap(data)?;
        RawSourceMapBundle::analyze(config, &sourcemap)
    }
}

fn unembed_sourcemap(input_data: &[u8]) -> anyhow::Result<Vec<u8>> {
    let ref_pos = input_data
        .windows(REF_MARKER.len())
        .position(|chunk| chunk == REF_MARKER)
        .ok_or_else(|| anyhow!("Must be called with data that contains a source map"))?;

    let sourcemap = &input_data[ref_pos..];

    const DATA_MARKER: &[u8] = b"base64,";
    let data_pos = sourcemap
        .windows(DATA_MARKER.len())
        .position(|chunk| chunk == DATA_MARKER)
        .ok_or_else(|| anyhow!("Sourcemap data URL is not base64"))?;
    let data = &sourcemap[(data_pos + DATA_MARKER.len())..];
    let end = data.iter().position(|c| *c == b'\n').unwrap_or(data.len());
    BASE64.decode(&data[..end]).context("Decoding base64")
}
