//! Programmatic screenshotting and measurement of HTML pages served by Stencila
//!
//! This crate provides functionality to:
//! - Capture screenshots of HTML pages using headless Chrome
//! - Measure computed CSS properties and layout metrics
//! - Evaluate assertions about measurements
//! - Discover and connect to running Stencila servers

mod assertions;
mod browser;
pub mod cli;
mod devices;
mod measure;
mod output;
mod server;

use assertions::{Assertion, AssertionResults};
use browser::{BrowserSession, CaptureOptions, ColorScheme, WaitConfig};
use devices::{DevicePreset, ViewportConfig};
use output::{SnapOutput, TargetInfo, Timings};
use server::ServerInfo;

use std::path::PathBuf;
use std::time::Instant;

/// Options for the snap operation
#[derive(Debug)]
struct SnapOptions {
    /// Path to document or directory (relative to server root)
    path: Option<PathBuf>,

    /// Override URL (instead of discovering server)
    url: Option<String>,

    /// Output screenshot path (.png)
    output: Option<PathBuf>,

    /// CSS selector to capture or measure
    selector: Option<String>,

    /// Capture full scrollable page
    full_page: bool,

    /// Device preset
    device: Option<DevicePreset>,

    /// Custom viewport configuration
    viewport: Option<ViewportConfig>,

    /// Color scheme override
    color_scheme: Option<ColorScheme>,

    /// Emulate print media
    print_media: bool,

    /// Wait configuration
    wait_config: WaitConfig,

    /// Collect computed CSS and layout metrics
    measure: bool,

    /// Assertions to evaluate
    assertions: Vec<String>,
}

/// Main entry point for snap operation
async fn snap(options: SnapOptions) -> eyre::Result<SnapOutput> {
    let start = Instant::now();

    let (url, server_handle) = if let Some(url) = options.url {
        (url, None)
    } else {
        let server = ServerInfo::discover(options.path.as_deref()).await?;
        let url = server.info.resolve_url(options.path)?;
        (url, Some(server))
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

    // Measure (if requested or if assertions present)
    let measurements = if options.measure || !options.assertions.is_empty() {
        Some(
            browser
                .inject_and_measure(options.selector.as_deref())
                .await?,
        )
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

    // Capture screenshot (if output specified)
    if let Some(output) = &options.output {
        let capture_opts = CaptureOptions {
            full_page: options.full_page,
            selector: options.selector.clone(),
        };
        browser
            .capture_screenshot(&capture_opts, output, &viewport)
            .await?;
    }

    // Build output
    let elapsed = start.elapsed();
    let ok = assertion_results.passed && assertion_results.failures.is_empty();

    // Gracefully shutdown the server
    if let Some(server) = server_handle
        && server.is_in_process()
    {
        server.shutdown().await?;
    }

    Ok(SnapOutput {
        ok,
        url,
        target: TargetInfo {
            selector: options.selector,
            full_page: options.full_page,
        },
        measure: measurements,
        assertions: assertion_results,
        timings: Timings {
            total_ms: elapsed.as_millis() as u64,
        },
    })
}
