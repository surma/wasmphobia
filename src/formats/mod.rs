pub mod dwarf;
pub mod sourcemaps;
pub mod wasm;

use std::collections::HashMap;

use anyhow::Context;
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
    fn name() -> String;
    fn can_handle(data: &[u8]) -> bool;
    fn analyze(config: &BundleAnalysisConfig, data: &[u8]) -> anyhow::Result<BundleAnalysis>;
}

impl<F1: BundleFormat, F2: BundleFormat> BundleFormat for (F1, F2) {
    fn name() -> String {
        format!("{}+{}", F1::name(), F2::name())
    }

    fn can_handle(data: &[u8]) -> bool {
        F1::can_handle(data) || F2::can_handle(data)
    }

    fn analyze(config: &BundleAnalysisConfig, data: &[u8]) -> anyhow::Result<BundleAnalysis> {
        if F1::can_handle(data) {
            Ok(F1::analyze(config, data).context(F1::name())?)
        } else {
            Ok(F2::analyze(config, data).context(F2::name())?)
        }
    }
}

#[macro_export]
macro_rules! analyze_bundle_with_formats {
    (@, $a:ident) => {
        $a
    };
    (@, $a:ident, $($f:ident),*) => {
        ($a, analyze_bundle_with_formats!(@, $($f),*))
    };
    ($c:expr, $v:expr,$($f:ident),*, ) => {
        analyze_bundle::<analyze_bundle_with_formats!(@, $($f),*)>($c, $v)
    };
}

pub fn analyze_bundle<T: BundleFormat>(
    config: &BundleAnalysisConfig,
    data: &[u8],
) -> anyhow::Result<BundleAnalysis> {
    T::analyze(config, data).context(T::name())
}
