//! Output structures for snap results

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::{assertions::AssertionResults, measure::MeasureResult};

/// Main output structure for snap operation
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct SnapOutput {
    /// Whether the operation succeeded (all assertions passed, no errors)
    pub ok: bool,

    /// URL that was captured
    pub url: String,

    /// Target information (selector, full_page)
    pub target: TargetInfo,

    /// Measurement results (if collected)
    pub measure: Option<MeasureResult>,

    /// Assertion evaluation results
    pub assertions: AssertionResults,

    /// Timing information
    pub timings: Timings,
}

impl SnapOutput {
    /// Serialize to pretty-printed JSON string
    pub fn to_json(&self) -> eyre::Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }
}

/// Information about what was targeted for capture/measurement
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct TargetInfo {
    /// CSS selector (if specific element targeted)
    pub selector: Option<String>,

    /// Whether full page was captured
    pub full_page: bool,
}

/// Timing information
#[derive(Debug, Serialize, Deserialize)]
pub struct Timings {
    /// Total time in milliseconds
    pub total_ms: u64,
}
