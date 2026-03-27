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

use std::{collections::BTreeMap, path::PathBuf, time::Instant};

use assertions::{Assertion, AssertionResults};
use browser::{BrowserSession, CaptureOptions};
use image::{
    GenericImageView, ImageFormat, ImageReader, codecs::png::PngEncoder, imageops::FilterType,
};
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

const MAX_SCREENSHOT_DIMENSION: u32 = 8_000;
const OPTIMIZED_SCREENSHOT_DIMENSION: u32 = 4_096;

/// Screenshot resizing strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenshotResizeMode {
    /// Preserve the captured image exactly as returned by the browser.
    Never,
    /// Resize only when the image exceeds the provider-safe hard limit.
    Auto,
    /// Resize to the configured max dimension even below hard limits.
    Optimize,
}

/// Screenshot resize configuration.
#[derive(Debug, Clone, Copy)]
pub struct ScreenshotResizePolicy {
    /// Strategy used to decide whether to resize.
    pub mode: ScreenshotResizeMode,
    /// Maximum allowed output dimension after resizing.
    pub max_dimension: Option<u32>,
}

impl Default for ScreenshotResizePolicy {
    fn default() -> Self {
        Self {
            mode: ScreenshotResizeMode::Auto,
            max_dimension: None,
        }
    }
}

impl ScreenshotResizePolicy {
    fn resolved_max_dimension(self) -> Option<u32> {
        match self.mode {
            ScreenshotResizeMode::Never => None,
            ScreenshotResizeMode::Auto => {
                Some(self.max_dimension.unwrap_or(MAX_SCREENSHOT_DIMENSION))
            }
            ScreenshotResizeMode::Optimize => {
                Some(self.max_dimension.unwrap_or(OPTIMIZED_SCREENSHOT_DIMENSION))
            }
        }
    }
}

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

    /// Screenshot resize policy applied after capture.
    pub screenshot_resize: ScreenshotResizePolicy,
}

fn resize_screenshot_if_needed(
    bytes: Vec<u8>,
    policy: ScreenshotResizePolicy,
) -> eyre::Result<(Vec<u8>, Option<output::ScreenshotResize>)> {
    let Some(max_dimension) = policy.resolved_max_dimension() else {
        return Ok((bytes, None));
    };

    let reader = ImageReader::with_format(std::io::Cursor::new(&bytes), ImageFormat::Png);
    let image = reader.decode()?;
    let (width, height) = image.dimensions();
    let largest_dimension = width.max(height);

    if largest_dimension <= max_dimension {
        return Ok((bytes, None));
    }

    let scale = max_dimension as f64 / largest_dimension as f64;
    let resized_width = ((width as f64 * scale).round() as u32).max(1);
    let resized_height = ((height as f64 * scale).round() as u32).max(1);
    let resized = image.resize(resized_width, resized_height, FilterType::Lanczos3);

    let mut output = Vec::new();
    let encoder = PngEncoder::new(&mut output);
    resized.write_with_encoder(encoder)?;

    let mode = match policy.mode {
        ScreenshotResizeMode::Never => "never",
        ScreenshotResizeMode::Auto => "auto",
        ScreenshotResizeMode::Optimize => "optimize",
    };

    let note = format!(
        "Screenshot was downscaled from {width}x{height} to {resized_width}x{resized_height} using {mode} mode with max dimension {max_dimension}px."
    );

    Ok((
        output,
        Some(output::ScreenshotResize {
            original_width: width,
            original_height: height,
            resized_width,
            resized_height,
            mode: mode.to_string(),
            max_dimension,
            note,
        }),
    ))
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

    let (screenshot_bytes, screenshot_resize) = if let Some(bytes) = screenshot_bytes {
        let (bytes, resize) = resize_screenshot_if_needed(bytes, options.screenshot_resize)?;
        (Some(bytes), resize)
    } else {
        (None, None)
    };

    // Multi-device batch (if --devices specified)
    let devices_result = if let Some(device_presets) = &options.devices {
        let mut device_results = BTreeMap::new();

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

            let (device_screenshot, device_screenshot_resize) =
                if let Some(bytes) = device_screenshot {
                    let (bytes, resize) =
                        resize_screenshot_if_needed(bytes, options.screenshot_resize)?;
                    (Some(bytes), resize)
                } else {
                    (None, None)
                };

            let device_name = format!("{device:?}").to_lowercase();
            device_results.insert(
                device_name,
                DeviceSnapResult {
                    viewport: device_viewport,
                    measure: device_measure,
                    screenshot: device_screenshot,
                    screenshot_resize: device_screenshot_resize,
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
        screenshot_resize,
        devices: devices_result,
        timings: Timings {
            total_ms: elapsed.as_millis() as u64,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ColorType, ImageBuffer, ImageEncoder, Rgb, codecs::png::PngEncoder};

    fn png_bytes(width: u32, height: u32) -> eyre::Result<Vec<u8>> {
        let mut bytes = Vec::new();
        let image =
            ImageBuffer::<Rgb<u8>, Vec<u8>>::from_pixel(width, height, Rgb([255, 255, 255]));
        PngEncoder::new(&mut bytes).write_image(
            image.as_raw(),
            width,
            height,
            ColorType::Rgb8.into(),
        )?;
        Ok(bytes)
    }

    #[test]
    fn auto_resize_downscales_when_over_hard_limit() -> eyre::Result<()> {
        let png = png_bytes(8_100, 2_000)?;
        let (resized, meta) = resize_screenshot_if_needed(png, ScreenshotResizePolicy::default())?;
        let meta = meta.expect("expected resize metadata");

        assert_eq!(meta.original_width, 8_100);
        assert_eq!(meta.resized_width, 8_000);
        assert!(!resized.is_empty());

        Ok(())
    }

    #[test]
    fn optimize_resize_downscales_below_hard_limit() -> eyre::Result<()> {
        let png = png_bytes(5_000, 2_500)?;
        let (resized, meta) = resize_screenshot_if_needed(
            png,
            ScreenshotResizePolicy {
                mode: ScreenshotResizeMode::Optimize,
                max_dimension: Some(4_096),
            },
        )?;
        let meta = meta.expect("expected resize metadata");

        assert_eq!(meta.original_width, 5_000);
        assert_eq!(meta.resized_width, 4_096);
        assert!(!resized.is_empty());

        Ok(())
    }

    #[test]
    fn never_resize_preserves_image() -> eyre::Result<()> {
        let png = png_bytes(8_100, 2_000)?;
        let (resized, meta) = resize_screenshot_if_needed(
            png.clone(),
            ScreenshotResizePolicy {
                mode: ScreenshotResizeMode::Never,
                max_dimension: None,
            },
        )?;

        assert!(meta.is_none());
        assert_eq!(resized, png);

        Ok(())
    }
}
