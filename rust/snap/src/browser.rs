//! Browser automation using headless Chrome

use std::{ffi::OsStr, path::Path, sync::Arc, thread::sleep, time::Duration};

use clap::ValueEnum;
use eyre::{Result, eyre};
use headless_chrome::{Browser, LaunchOptionsBuilder, Tab};
use serde::{Deserialize, Serialize};
use tokio::fs;

use crate::{
    devices::ViewportConfig,
    measure::{MEASUREMENT_SCRIPT, MeasureResult, parse_measurements},
};

/// Wait condition for page load events
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum WaitUntil {
    /// Wait for 'load' event
    Load,
    /// Wait for 'DOMContentLoaded' event
    DomContentLoaded,
    /// Wait for network idle (default)
    NetworkIdle,
}

impl Default for WaitUntil {
    fn default() -> Self {
        Self::NetworkIdle
    }
}

/// Color scheme preference for rendering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum ColorScheme {
    /// Light color scheme
    Light,
    /// Dark color scheme
    Dark,
    /// System default (no override)
    System,
}

impl Default for ColorScheme {
    fn default() -> Self {
        Self::System
    }
}

/// Wait configuration
#[derive(Debug, Clone, Default)]
pub struct WaitConfig {
    /// Wait condition: load, domcontentloaded, or networkidle
    pub wait_until: WaitUntil,

    /// Wait for specific CSS selector
    pub wait_for: Option<String>,

    /// Additional delay in milliseconds
    pub delay: Option<u64>,
}

/// Options for capturing screenshots
#[derive(Debug, Clone)]
pub struct CaptureOptions {
    /// Capture full scrollable page
    pub full_page: bool,

    /// CSS selector to capture (if specific element)
    pub selector: Option<String>,
}

/// Create a browser instance with optimized options
fn create_browser() -> Result<Browser> {
    let args: Vec<&OsStr> = [
        "--disable-dev-shm-usage",
        "--disable-extensions",
        "--disable-gpu",
        "--disable-hang-monitor",
        "--disable-renderer-backgrounding",
        "--force-device-scale-factor=1",
        "--memory-pressure-off",
        "--no-first-run",
        "--no-default-browser-check",
        "--disable-backgrounding-occluded-windows",
        "--disable-breakpad",
    ]
    .iter()
    .map(OsStr::new)
    .collect();

    let options = LaunchOptionsBuilder::default()
        .headless(true)
        .args(args)
        .build()
        .map_err(|error| eyre!("Failed to build launch options: {error}"))?;

    Browser::new(options).map_err(|error| eyre!("Failed to launch browser: {error}"))
}

/// Browser session for navigation and measurement
pub struct BrowserSession {
    #[allow(dead_code)]
    browser: Browser,
    tab: Arc<Tab>,
}

impl BrowserSession {
    /// Create a new browser session with the specified viewport and media settings
    pub fn new(viewport: ViewportConfig, print_media: bool) -> Result<Self> {
        let browser = create_browser()?;
        let tab = browser
            .new_tab()
            .map_err(|error| eyre!("Failed to create tab: {error}"))?;

        // Set viewport using Bounds type
        tab.set_bounds(headless_chrome::types::Bounds::Normal {
            left: None,
            top: None,
            width: Some(viewport.width as f64),
            height: Some(viewport.height as f64),
        })
        .map_err(|error| eyre!("Failed to set bounds: {error}"))?;

        // Set device scale factor (DPR)
        tab.call_method(
            headless_chrome::protocol::cdp::Emulation::SetDeviceMetricsOverride {
                width: viewport.width,
                height: viewport.height,
                device_scale_factor: viewport.dpr as f64,
                mobile: false,
                scale: None,
                screen_width: None,
                screen_height: None,
                position_x: None,
                position_y: None,
                dont_set_visible_size: None,
                screen_orientation: None,
                viewport: None,
                display_feature: None,
                device_posture: None,
            },
        )
        .map_err(|error| eyre!("Failed to set device metrics: {error}"))?;

        // Set emulated media (print media type and/or color scheme)
        let needs_media_emulation = print_media || viewport.color_scheme != ColorScheme::System;

        if needs_media_emulation {
            let media = if print_media {
                Some("print".to_string())
            } else {
                None
            };

            let features = if viewport.color_scheme != ColorScheme::System {
                let color_value = match viewport.color_scheme {
                    ColorScheme::Light => "light",
                    ColorScheme::Dark => "dark",
                    ColorScheme::System => unreachable!(),
                };

                Some(vec![
                    headless_chrome::protocol::cdp::Emulation::MediaFeature {
                        name: "prefers-color-scheme".to_string(),
                        value: color_value.to_string(),
                    },
                ])
            } else {
                None
            };

            tab.call_method(
                headless_chrome::protocol::cdp::Emulation::SetEmulatedMedia { media, features },
            )
            .map_err(|error| eyre!("Failed to set emulated media: {error}"))?;
        }

        Ok(Self { browser, tab })
    }

    /// Navigate to URL and wait according to configuration
    pub async fn navigate_and_wait(
        &mut self,
        url: &str,
        wait_config: &WaitConfig,
        print_media: bool,
    ) -> Result<()> {
        // Navigate to URL
        self.tab
            .navigate_to(url)
            .map_err(|error| eyre!("Failed to navigate to {url}: {error}"))?;

        // Wait based on configuration
        match wait_config.wait_until {
            WaitUntil::Load | WaitUntil::DomContentLoaded => {
                self.tab
                    .wait_for_element("body")
                    .map_err(|error| eyre!("Failed to wait for body: {error}"))?;
            }
            WaitUntil::NetworkIdle => {
                // Wait for network idle
                self.tab
                    .wait_until_navigated()
                    .map_err(|error| eyre!("Failed to wait for navigation: {error}"))?;
                // Additional delay for dynamic content
                sleep(Duration::from_millis(500));
            }
        }

        // If print media is enabled, inject CSS to simulate @page margins
        // Based on --page-margin-* and --page-padding-* from web/src/themes/base/pages.css:
        // - Top: 1.5cm margin + 0.5cm padding = 2cm (~75px at 96 DPI)
        // - Bottom: 1.5cm margin + 0.5cm padding = 2cm (~75px at 96 DPI)
        // - Left/Right: 2cm margin (~75px at 96 DPI)
        if print_media {
            let inject_margins_script = r#"
                document.documentElement.style.padding = '75px';
                document.documentElement.style.overflow = 'hidden';
            "#;

            self.tab
                .evaluate(inject_margins_script, false)
                .map_err(|error| eyre!("Failed to inject print margins: {error}"))?;
        }

        // Wait for specific selector if provided
        if let Some(selector) = &wait_config.wait_for {
            self.tab
                .wait_for_element(selector)
                .map_err(|error| eyre!("Failed to wait for selector {selector}: {error}"))?;
        }

        // Additional delay if specified
        if let Some(delay_ms) = wait_config.delay {
            sleep(Duration::from_millis(delay_ms));
        }

        Ok(())
    }

    /// Inject measurement script and collect results
    pub async fn inject_and_measure(&mut self, selector: Option<&str>) -> Result<MeasureResult> {
        let selector_arg = selector
            .map(|s| format!("'{}'", s))
            .unwrap_or_else(|| "null".to_string());

        // Don't use semicolon at end - we want the value returned from the IIFE
        let script = format!("({MEASUREMENT_SCRIPT})({selector_arg})");

        let result = self
            .tab
            .evaluate(&script, false)
            .map_err(|error| eyre!("Failed to evaluate measurement script: {error}"))?;

        // Check if result is null or None and provide better error message
        let value = result.value.ok_or_else(|| {
            eyre!("Measurement script returned no value. This might indicate a JavaScript error or that the page is not ready.")
        })?;

        if value.is_null() {
            return Err(eyre!(
                "Measurement script returned null. This may indicate that the document is not fully loaded or the JavaScript failed to execute."
            ));
        }

        let json_str = serde_json::to_string(&value)?;
        parse_measurements(&json_str)
    }

    /// Capture screenshot
    pub async fn capture_screenshot(
        &mut self,
        options: &CaptureOptions,
        path: &Path,
        viewport: &ViewportConfig,
    ) -> Result<()> {
        let screenshot_data = if let Some(selector) = &options.selector {
            // Element-specific screenshot
            let element = self
                .tab
                .wait_for_element(selector)
                .map_err(|error| eyre!("Failed to find selector {selector}: {error}"))?;
            element
                .capture_screenshot(
                    headless_chrome::protocol::cdp::Page::CaptureScreenshotFormatOption::Png,
                )
                .map_err(|error| eyre!("Failed to capture element screenshot: {error}"))?
        } else if options.full_page {
            // For full-page screenshots, we need to get the actual content height
            // and adjust the viewport accordingly to capture everything
            let content_height_script = r#"
                Math.max(
                    document.body.scrollHeight,
                    document.body.offsetHeight,
                    document.documentElement.clientHeight,
                    document.documentElement.scrollHeight,
                    document.documentElement.offsetHeight
                )
            "#;

            let result = self
                .tab
                .evaluate(content_height_script, false)
                .map_err(|error| eyre!("Failed to get content height: {error}"))?;

            let content_height = result
                .value
                .and_then(|v| v.as_f64())
                .ok_or_else(|| eyre!("Failed to parse content height"))?
                as u32;

            // Update viewport height to match content height
            self.tab
                .call_method(
                    headless_chrome::protocol::cdp::Emulation::SetDeviceMetricsOverride {
                        width: viewport.width,
                        height: content_height,
                        device_scale_factor: viewport.dpr as f64,
                        mobile: false,
                        scale: None,
                        screen_width: None,
                        screen_height: None,
                        position_x: None,
                        position_y: None,
                        dont_set_visible_size: None,
                        screen_orientation: None,
                        viewport: None,
                        display_feature: None,
                        device_posture: None,
                    },
                )
                .map_err(|error| eyre!("Failed to update viewport for full page: {error}"))?;

            // Small delay to let the viewport adjustment take effect
            sleep(Duration::from_millis(100));

            // Now capture the full page
            self.tab
                .capture_screenshot(
                    headless_chrome::protocol::cdp::Page::CaptureScreenshotFormatOption::Png,
                    None,
                    None,
                    true,
                )
                .map_err(|error| eyre!("Failed to capture full page screenshot: {error}"))?
        } else {
            // Viewport-only screenshot
            self.tab
                .capture_screenshot(
                    headless_chrome::protocol::cdp::Page::CaptureScreenshotFormatOption::Png,
                    None,
                    None,
                    false,
                )
                .map_err(|error| eyre!("Failed to capture viewport screenshot: {error}"))?
        };

        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir).await?;
        }
        fs::write(path, screenshot_data).await?;

        Ok(())
    }
}
