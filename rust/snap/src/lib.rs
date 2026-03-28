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
use browser::{BrowserSession, CaptureOptions, CaptureResult};
use image::{
    GenericImageView, ImageFormat, ImageReader, codecs::png::PngEncoder, imageops::FilterType,
};
use measure::{MeasurementContext, enrich_measurements, selectors_for_preset};

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
use output::{
    CaptureInfo, CaptureMode, DeviceSnapResult, SnapOutput, TargetInfo, Timings, TokenOutput,
};
use server::ServerInfo;

const MAX_SCREENSHOT_DIMENSION: u32 = 8_000;
const OPTIMIZED_SCREENSHOT_DIMENSION: u32 = 4_096;
const LONG_PAGE_WARNING_HEIGHT: u32 = 12_000;
const SMALL_RESIZE_SCALE_WARNING: f64 = 0.5;

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

    /// Filter extracted tokens to matching prefixes
    pub token_prefixes: Vec<String>,

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

fn measure_preset_name(preset: MeasurePreset) -> String {
    match preset {
        MeasurePreset::Document => "document",
        MeasurePreset::Site => "site",
        MeasurePreset::All => "all",
        MeasurePreset::Header => "header",
        MeasurePreset::Nav => "nav",
        MeasurePreset::Main => "main",
        MeasurePreset::Footer => "footer",
        MeasurePreset::Theme => "theme",
    }
    .to_string()
}

fn normalize_token_prefix(prefix: &str) -> String {
    if prefix.starts_with("--") {
        prefix.to_string()
    } else {
        format!("--{prefix}")
    }
}

fn filter_and_group_tokens(tokens: BTreeMap<String, String>, prefixes: &[String]) -> TokenOutput {
    let normalized_prefixes: Vec<String> = prefixes
        .iter()
        .map(|prefix| normalize_token_prefix(prefix))
        .collect();

    let values: BTreeMap<String, String> = tokens
        .into_iter()
        .filter(|(name, _)| {
            normalized_prefixes.is_empty()
                || normalized_prefixes
                    .iter()
                    .any(|prefix| name.starts_with(prefix))
        })
        .collect();

    let mut groups: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();
    for (name, value) in &values {
        let family = name
            .trim_start_matches('-')
            .split('-')
            .next()
            .filter(|family| !family.is_empty())
            .unwrap_or("other")
            .to_string();
        groups
            .entry(family)
            .or_default()
            .insert(name.clone(), value.clone());
    }

    TokenOutput {
        values,
        groups,
        prefixes: (!normalized_prefixes.is_empty()).then_some(normalized_prefixes),
    }
}

fn build_capture_info(
    mode: CaptureMode,
    selector: Option<&String>,
    matched_elements: Option<usize>,
    full_page_content_height: Option<u32>,
    screenshot_resize: Option<&output::ScreenshotResize>,
) -> CaptureInfo {
    let mut diagnostics = Vec::new();

    if let Some(content_height) = full_page_content_height
        && content_height > LONG_PAGE_WARNING_HEIGHT
    {
        diagnostics.push(format!(
            "Full-page content height is {content_height}px; the resulting image may be difficult to review without region-focused captures."
        ));
    }

    if let Some(resize) = screenshot_resize {
        let width_scale = resize.resized_width as f64 / resize.original_width as f64;
        let height_scale = resize.resized_height as f64 / resize.original_height as f64;
        let scale = width_scale.min(height_scale);
        if scale < SMALL_RESIZE_SCALE_WARNING {
            diagnostics.push(format!(
                "Screenshot was downscaled to {:.0}% of its original size, which may reduce readability for typography and spacing review.",
                scale * 100.0
            ));
        }
    }

    CaptureInfo {
        mode,
        used_selector_for_capture: matches!(mode, CaptureMode::Element),
        selector: selector.cloned(),
        matched_elements,
        full_page_content_height,
        diagnostics,
    }
}

/// Normalize an optional string: treat empty and whitespace-only values as `None`.
fn normalize_optional_selector(value: Option<String>) -> Option<String> {
    value.filter(|s| !s.trim().is_empty())
}

/// Main entry point for snap operation
pub async fn snap(options: SnapOptions) -> eyre::Result<SnapOutput> {
    let start = Instant::now();

    // Normalize selector and wait_for: treat empty/whitespace-only strings as None
    // so that callers don't need to distinguish between absent and empty values.
    let selector = normalize_optional_selector(options.selector);
    let wait_config = WaitConfig {
        wait_for: normalize_optional_selector(options.wait_config.wait_for),
        wait_until: options.wait_config.wait_until,
        delay: options.wait_config.delay,
    };

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
        .navigate_and_wait(&url, &wait_config, options.print_media)
        .await?;

    // Resolve measure mode to a concrete preset.
    // Assertions implicitly enable measurement when mode is Off; use `All` so
    // that selectors from both `Document` and `Site` presets are available for
    // assertion evaluation (the previous route-dependent default meant that
    // e.g. `css(stencila-paragraph)` assertions failed on site routes because
    // the `Site` preset does not include document-content selectors).
    let resolved_preset = match &options.measure {
        MeasureMode::Off if options.assertions.is_empty() => None,
        MeasureMode::Off if !options.assertions.is_empty() => Some(MeasurePreset::All),
        MeasureMode::Off | MeasureMode::Auto => Some(match &target {
            ResolvedTarget::Route(_) => MeasurePreset::Site,
            ResolvedTarget::Path(_) => MeasurePreset::Document,
        }),
        MeasureMode::Preset(p) => Some(*p),
    };

    // Build selector list
    let selector_overrides_preset = selector.is_some();
    let mut selectors = if let Some(sel) = &selector {
        // Explicit selector overrides preset
        vec![sel.clone()]
    } else if let Some(preset) = resolved_preset {
        selectors_for_preset(preset)
    } else {
        vec![]
    };

    // Parse assertions once up front so that parse errors are reported
    // immediately and the parsed values can be reused for both selector
    // collection and later evaluation.
    let parsed_assertions: Vec<Assertion> = options
        .assertions
        .iter()
        .map(|s| Assertion::parse(s))
        .collect::<eyre::Result<Vec<_>>>()?;

    // Ensure assertion selectors are measured even if they aren't in the preset
    for assertion in &parsed_assertions {
        if !selectors.contains(&assertion.selector) {
            selectors.push(assertion.selector.clone());
        }
    }

    // Measure (if selectors are non-empty)
    let measurements = if !selectors.is_empty() {
        let mut measurements = browser.inject_and_measure(&selectors).await?;
        enrich_measurements(
            &mut measurements,
            &viewport,
            MeasurementContext {
                viewport_only_capture: options.screenshot
                    && selector.is_none()
                    && !options.full_page,
            },
        );
        Some(measurements)
    } else {
        None
    };

    // Extract tokens (if requested)
    let tokens = if options.tokens {
        Some(filter_and_group_tokens(
            browser.inject_tokens().await?,
            &options.token_prefixes,
        ))
    } else {
        None
    };

    // Extract palette (if requested)
    let palette = if options.palette {
        Some(browser.inject_palette().await?)
    } else {
        None
    };

    // Evaluate assertions using the already-parsed values
    let assertion_results = if !parsed_assertions.is_empty() {
        let measurements = measurements
            .as_ref()
            .ok_or_else(|| eyre::eyre!("Measurements required for assertions but not collected"))?;

        AssertionResults::evaluate(&parsed_assertions, measurements)?
    } else {
        AssertionResults::default()
    };

    // Capture screenshot bytes (if requested)
    let raw_capture = if options.screenshot {
        let capture_opts = CaptureOptions {
            full_page: options.full_page,
            selector: selector.clone(),
        };
        Some(browser.capture_screenshot(&capture_opts, &viewport).await?)
    } else {
        None
    };

    let (screenshot_bytes, screenshot_resize, matched_elements, full_page_content_height) =
        if let Some(CaptureResult {
            bytes,
            matched_elements,
            full_page_content_height,
        }) = raw_capture
        {
            let (bytes, resize) = resize_screenshot_if_needed(bytes, options.screenshot_resize)?;
            (
                Some(bytes),
                resize,
                matched_elements,
                full_page_content_height,
            )
        } else {
            (None, None, None, None)
        };

    let capture = if options.screenshot {
        let mode = if selector.is_some() {
            CaptureMode::Element
        } else if options.full_page {
            CaptureMode::FullPage
        } else {
            CaptureMode::Viewport
        };

        Some(build_capture_info(
            mode,
            selector.as_ref(),
            matched_elements,
            full_page_content_height,
            screenshot_resize.as_ref(),
        ))
    } else {
        None
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
                .navigate_and_wait(&url, &wait_config, options.print_media)
                .await?;

            // Measure with same selectors
            let device_measure = if !selectors.is_empty() {
                let mut measurements = browser.inject_and_measure(&selectors).await?;
                enrich_measurements(
                    &mut measurements,
                    &device_viewport,
                    MeasurementContext {
                        viewport_only_capture: options.screenshot
                            && selector.is_none()
                            && !options.full_page,
                    },
                );
                Some(measurements)
            } else {
                None
            };

            // Capture device screenshot bytes (if requested)
            let device_capture = if options.screenshot {
                let capture_opts = CaptureOptions {
                    full_page: options.full_page,
                    selector: selector.clone(),
                };
                Some(
                    browser
                        .capture_screenshot(&capture_opts, &device_viewport)
                        .await?,
                )
            } else {
                None
            };

            let (
                device_screenshot,
                device_screenshot_resize,
                device_matched_elements,
                device_full_page_content_height,
            ) = if let Some(CaptureResult {
                bytes,
                matched_elements,
                full_page_content_height,
            }) = device_capture
            {
                let (bytes, resize) =
                    resize_screenshot_if_needed(bytes, options.screenshot_resize)?;
                (
                    Some(bytes),
                    resize,
                    matched_elements,
                    full_page_content_height,
                )
            } else {
                (None, None, None, None)
            };

            let device_capture = if options.screenshot {
                Some(build_capture_info(
                    if selector.is_some() {
                        CaptureMode::Element
                    } else if options.full_page {
                        CaptureMode::FullPage
                    } else {
                        CaptureMode::Viewport
                    },
                    selector.as_ref(),
                    device_matched_elements,
                    device_full_page_content_height,
                    device_screenshot_resize.as_ref(),
                ))
            } else {
                None
            };

            let device_name = format!("{device:?}").to_lowercase();
            device_results.insert(
                device_name,
                DeviceSnapResult {
                    viewport: device_viewport,
                    measure: device_measure,
                    capture: device_capture,
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
            selector,
            full_page: options.full_page,
            measured_selectors: selectors,
            measure_preset: if !selector_overrides_preset {
                resolved_preset.map(measure_preset_name)
            } else {
                None
            },
        },
        capture,
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

    #[test]
    fn build_capture_info_preserves_selector_match_count_without_measurements() {
        let capture = build_capture_info(
            CaptureMode::Element,
            Some(&".title".to_string()),
            Some(3),
            None,
            None,
        );

        assert_eq!(capture.matched_elements, Some(3));
        assert!(capture.used_selector_for_capture);
    }

    #[test]
    fn normalize_optional_selector_filters_empty_values() {
        assert_eq!(normalize_optional_selector(None), None);
        assert_eq!(normalize_optional_selector(Some(String::new())), None);
        assert_eq!(normalize_optional_selector(Some("   ".to_string())), None);
        assert_eq!(
            normalize_optional_selector(Some("body".to_string())),
            Some("body".to_string())
        );
        assert_eq!(
            normalize_optional_selector(Some(" .title ".to_string())),
            Some(" .title ".to_string())
        );
    }
}
