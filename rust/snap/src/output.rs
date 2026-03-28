//! Output structures for snap results

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::{assertions::AssertionResults, measure::MeasureResult};

/// How the screenshot was captured.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CaptureMode {
    Viewport,
    FullPage,
    Element,
}

/// Information about screenshot resizing performed after capture.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenshotResize {
    /// Width before resizing.
    pub original_width: u32,

    /// Height before resizing.
    pub original_height: u32,

    /// Width after resizing.
    pub resized_width: u32,

    /// Height after resizing.
    pub resized_height: u32,

    /// Resize mode that was applied.
    pub mode: String,

    /// Maximum dimension target used for the resize.
    pub max_dimension: u32,

    /// Human-readable note describing the resize.
    pub note: String,
}

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

    /// Information about screenshot capture semantics
    pub capture: Option<CaptureInfo>,

    /// Measurement results (if collected)
    pub measure: Option<MeasureResult>,

    /// Resolved CSS custom property (token) values (if `--tokens` used)
    pub tokens: Option<TokenOutput>,

    /// Color palette extracted from the page (if `--palette` used)
    pub palette: Option<Vec<PaletteEntry>>,

    /// Assertion evaluation results
    pub assertions: AssertionResults,

    /// Screenshot PNG bytes (if captured). Skipped during serialization
    /// because JSON output should not contain large binary blobs.
    #[serde(skip)]
    pub screenshot: Option<Vec<u8>>,

    /// Metadata about screenshot resizing, when applied.
    pub screenshot_resize: Option<ScreenshotResize>,

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

    /// How many elements the explicit selector matched (from measurements).
    /// Present even when `screenshot` is false, so agents can check element
    /// existence without requesting a screenshot.
    pub selector_matched: Option<usize>,

    /// Whether full page was captured
    pub full_page: bool,

    /// Concrete selectors measured during this run
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub measured_selectors: Vec<String>,

    /// Measurement preset that was selected, if any
    pub measure_preset: Option<String>,
}

/// Screenshot capture metadata.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureInfo {
    /// How the screenshot was captured
    pub mode: CaptureMode,

    /// Whether the selector controlled screenshot capture
    pub used_selector_for_capture: bool,

    /// The selector used for capture, if any
    pub selector: Option<String>,

    /// How many elements the selector matched
    pub matched_elements: Option<usize>,

    /// Full-page document height used during capture
    pub full_page_content_height: Option<u32>,

    /// Capture-specific diagnostics
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub diagnostics: Vec<String>,
}

/// Token extraction output, including grouping and applied filters.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenOutput {
    /// Filtered token values keyed by CSS custom property name
    pub values: BTreeMap<String, String>,

    /// Tokens grouped by family prefix
    #[serde(skip_serializing_if = "BTreeMap::is_empty", default)]
    pub groups: BTreeMap<String, BTreeMap<String, String>>,

    /// Prefix filters that were applied
    pub prefixes: Option<Vec<String>>,
}

/// A color entry in the extracted palette
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaletteEntry {
    /// Hex color value (e.g., "#1f2937")
    pub hex: String,

    /// Number of elements using this color
    pub count: usize,
}

/// Timing information
#[derive(Debug, Serialize, Deserialize)]
pub struct Timings {
    /// Total time in milliseconds
    pub total_ms: u64,
}
