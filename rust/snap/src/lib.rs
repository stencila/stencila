//! Programmatic screenshotting and measurement of HTML pages served by Stencila
//!
//! This crate provides functionality to:
//! - Capture screenshots of HTML pages using headless Chrome
//! - Measure computed CSS properties and layout metrics
//! - Extract resolved CSS custom property (token) values
//! - Extract the page's color palette
//! - Evaluate assertions about measurements
//! - Discover and connect to running Stencila servers
//! - Batch-measure across multiple device viewports

mod assertions;
mod browser;
pub mod cli;
mod devices;
mod measure;
pub mod output;
mod server;

pub use browser::{ColorScheme, WaitConfig, WaitUntil};
pub use devices::{DevicePreset, ViewportConfig};
pub use measure::MeasurePreset;

use std::{collections::HashMap, path::PathBuf, time::Instant};

use assertions::{Assertion, AssertionResults};
use browser::{BrowserSession, CaptureOptions};
use measure::selectors_for_preset;

/// Whether and how to measure
#[derive(Debug)]
pub enum MeasureMode {
    /// No measurement
    Off,
    /// Auto-select preset based on target type
    Auto,
    /// Use a specific preset
    Preset(MeasurePreset),
}
use output::{DeviceSnapResult, SnapOutput, TargetInfo, Timings};
use server::ServerInfo;

/// Resolved target: either a site route or a filesystem path
enum ResolvedTarget {
    /// A site route like "/" or "/docs/guide/"
    Route(String),
    /// A filesystem path like "./my-doc.md"
    Path(PathBuf),
}

/// Options for the snap operation
#[derive(Debug)]
pub struct SnapOptions {
    /// Route or path (unified positional arg). Defaults to "/" when None.
    pub route_or_path: Option<String>,

    /// Override URL (instead of discovering server)
    pub url: Option<String>,

    /// Whether to capture a screenshot and return the PNG bytes in the output
    pub screenshot: bool,

    /// CSS selector to capture or measure
    pub selector: Option<String>,

    /// Capture full scrollable page
    pub full_page: bool,

    /// Single device preset
    pub device: Option<DevicePreset>,

    /// Multiple device presets for batch mode
    pub devices: Option<Vec<DevicePreset>>,

    /// Custom viewport configuration
    pub viewport: Option<ViewportConfig>,

    /// Color scheme override
    pub color_scheme: Option<ColorScheme>,

    /// Emulate print media
    pub print_media: bool,

    /// Wait configuration
    pub wait_config: WaitConfig,

    /// Measurement mode
    pub measure: MeasureMode,

    /// Extract resolved CSS custom property values
    pub tokens: bool,

    /// Extract the page's color palette
    pub palette: bool,

    /// Assertions to evaluate
    pub assertions: Vec<String>,
}

/// Main entry point for snap operation
pub async fn snap(options: SnapOptions) -> eyre::Result<SnapOutput> {
    let start = Instant::now();

    // Resolve the target: route or file path
    let target = match &options.route_or_path {
        None => ResolvedTarget::Route("/".to_string()),
        Some(value) => {
            let as_path = PathBuf::from(value);
            if as_path.is_file() {
                ResolvedTarget::Path(as_path)
            } else {
                ResolvedTarget::Route(value.clone())
            }
        }
    };

    // Discover server and resolve URL
    let url = if let Some(url) = options.url {
        url
    } else {
        match &target {
            ResolvedTarget::Route(route) => {
                let server = ServerInfo::discover(None, true).await?;
                server.resolve_route(route)
            }
            ResolvedTarget::Path(path) => {
                let server = ServerInfo::discover(Some(path), false).await?;
                server.resolve_url(Some(path.clone()))?
            }
        }
    };

    // Determine viewport configuration
    let mut viewport = if let Some(viewport) = options.viewport {
        viewport
    } else if let Some(device) = options.device {
        device.viewport()
    } else {
        ViewportConfig::default()
    };

    // Override color scheme if specified
    if let Some(color_scheme) = options.color_scheme {
        viewport.color_scheme = color_scheme;
    }

    // Initialize browser session
    let mut browser = BrowserSession::new(viewport, options.print_media)?;

    // Navigate and wait
    browser
        .navigate_and_wait(&url, &options.wait_config, options.print_media)
        .await?;

    // Resolve measure mode to a concrete preset.
    // Assertions implicitly enable measurement when mode is Off.
    let resolved_preset = match &options.measure {
        MeasureMode::Off if options.assertions.is_empty() => None,
        MeasureMode::Off | MeasureMode::Auto => Some(match &target {
            ResolvedTarget::Route(_) => MeasurePreset::Site,
            ResolvedTarget::Path(_) => MeasurePreset::Document,
        }),
        MeasureMode::Preset(p) => Some(*p),
    };

    // Build selector list
    let selectors = if let Some(sel) = &options.selector {
        // Explicit selector overrides preset
        vec![sel.clone()]
    } else if let Some(preset) = resolved_preset {
        selectors_for_preset(preset)
    } else {
        vec![]
    };

    // Measure (if selectors are non-empty)
    let measurements = if !selectors.is_empty() {
        Some(browser.inject_and_measure(&selectors).await?)
    } else {
        None
    };

    // Extract tokens (if requested)
    let tokens = if options.tokens {
        Some(browser.inject_tokens().await?)
    } else {
        None
    };

    // Extract palette (if requested)
    let palette = if options.palette {
        Some(browser.inject_palette().await?)
    } else {
        None
    };

    // Evaluate assertions
    let assertion_results = if !options.assertions.is_empty() {
        let assertions = options
            .assertions
            .iter()
            .map(|s| Assertion::parse(s))
            .collect::<eyre::Result<Vec<_>>>()?;

        let measurements = measurements
            .as_ref()
            .ok_or_else(|| eyre::eyre!("Measurements required for assertions but not collected"))?;

        AssertionResults::evaluate(&assertions, measurements)?
    } else {
        AssertionResults::default()
    };

    // Capture screenshot bytes (if requested)
    let screenshot_bytes = if options.screenshot {
        let capture_opts = CaptureOptions {
            full_page: options.full_page,
            selector: options.selector.clone(),
        };
        Some(browser.capture_screenshot(&capture_opts, &viewport).await?)
    } else {
        None
    };

    // Multi-device batch (if --devices specified)
    let devices_result = if let Some(device_presets) = &options.devices {
        let mut device_results = HashMap::new();

        for device in device_presets {
            let mut device_viewport = device.viewport();
            // Propagate color scheme override so --dark/--light apply to all devices
            if let Some(color_scheme) = options.color_scheme {
                device_viewport.color_scheme = color_scheme;
            }
            browser.resize_viewport(&device_viewport, options.print_media)?;

            // Reload and wait
            browser
                .navigate_and_wait(&url, &options.wait_config, options.print_media)
                .await?;

            // Measure with same selectors
            let device_measure = if !selectors.is_empty() {
                Some(browser.inject_and_measure(&selectors).await?)
            } else {
                None
            };

            // Capture device screenshot bytes (if requested)
            let device_screenshot = if options.screenshot {
                let capture_opts = CaptureOptions {
                    full_page: options.full_page,
                    selector: options.selector.clone(),
                };
                Some(
                    browser
                        .capture_screenshot(&capture_opts, &device_viewport)
                        .await?,
                )
            } else {
                None
            };

            let device_name = format!("{device:?}").to_lowercase();
            device_results.insert(
                device_name,
                DeviceSnapResult {
                    viewport: device_viewport,
                    measure: device_measure,
                    screenshot: device_screenshot,
                },
            );
        }

        Some(device_results)
    } else {
        None
    };

    // Close browser cleanly to avoid temp directory warnings
    browser.close();

    // Build output
    let elapsed = start.elapsed();
    let ok = assertion_results.passed;

    Ok(SnapOutput {
        ok,
        url,
        target: TargetInfo {
            selector: options.selector,
            full_page: options.full_page,
        },
        measure: measurements,
        tokens,
        palette,
        assertions: assertion_results,
        screenshot: screenshot_bytes,
        devices: devices_result,
        timings: Timings {
            total_ms: elapsed.as_millis() as u64,
        },
    })
}
