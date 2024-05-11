use std::collections::HashMap;

use anyhow::{anyhow, Context};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};

use crate::Args;

const MARKER: &[u8] = b"//# sourceMappingURL=data:";

pub fn unembed_sourcemap(input_data: &[u8]) -> anyhow::Result<Vec<u8>> {
    let ref_pos = input_data
        .windows(MARKER.len())
        .position(|chunk| chunk == MARKER)
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

pub fn has_embedded_sourcemap(input_data: &[u8]) -> bool {
    input_data
        .windows(MARKER.len())
        .any(|chunk| chunk == MARKER)
}

pub fn is_sourcemap(input_data: &[u8]) -> bool {
    const MARKER: &[u8] = b"\"mappings\"";
    if input_data[0] != b'{' {
        return false;
    }
    input_data
        .windows(MARKER.len())
        .any(|chunk| chunk == MARKER)
}

pub fn analyze_sourcemaps(_args: &Args, input_data: &[u8]) -> anyhow::Result<HashMap<String, u64>> {
    use sourcemap::SourceMap;
    let sm = SourceMap::from_slice(input_data)?;
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
