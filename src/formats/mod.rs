pub mod sourcemaps;
pub mod wasm;

use std::collections::HashMap;

use derive_more::{Deref, DerefMut};

use crate::Args;

#[derive(Debug, Clone, Default)]
pub struct BundleAnalysisConfig {
    retain_debug_sections: bool,
    files_only: bool,
    raw_symbols: bool,
}

impl From<Args> for BundleAnalysisConfig {
    fn from(value: Args) -> Self {
        BundleAnalysisConfig {
            retain_debug_sections: value.show_debug_sections,
            files_only: value.files_only,
            raw_symbols: value.raw_symbols,
        }
    }
}

#[derive(Debug, Clone, Deref, DerefMut, Default)]
pub struct BundleAnalysis(HashMap<String, u64>);

pub trait BundleFormat {
    fn can_handle(data: &[u8]) -> bool;
    fn analyze(config: &BundleAnalysisConfig, data: &[u8]) -> anyhow::Result<BundleAnalysis>;
}

impl<F1: BundleFormat, F2: BundleFormat> BundleFormat for (F1, F2) {
    fn can_handle(data: &[u8]) -> bool {
        F1::can_handle(data) || F2::can_handle(data)
    }

    fn analyze(config: &BundleAnalysisConfig, data: &[u8]) -> anyhow::Result<BundleAnalysis> {
        if F1::can_handle(data) {
            Ok(F1::analyze(config, data)?)
        } else {
            Ok(F2::analyze(config, data)?)
        }
    }
}

impl<F1: BundleFormat, F2: BundleFormat, F3: BundleFormat> BundleFormat for (F1, F2, F3) {
    fn can_handle(data: &[u8]) -> bool {
        F1::can_handle(data) || F2::can_handle(data)
    }

    fn analyze(config: &BundleAnalysisConfig, data: &[u8]) -> anyhow::Result<BundleAnalysis> {
        if F1::can_handle(data) {
            Ok(F1::analyze(config, data)?)
        } else {
            Ok(F2::analyze(config, data)?)
        }
    }
}

pub fn analyze_bundle<T: BundleFormat>(
    config: &BundleAnalysisConfig,
    data: &[u8],
) -> anyhow::Result<BundleAnalysis> {
    T::analyze(config, data)
}
