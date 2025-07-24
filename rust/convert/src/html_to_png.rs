//! HTML to PNG conversion using headless Chrome
//!
//! This module provides HTML to PNG conversion using a persistent browser
//! instance for optimal performance. Subsequent calls (~380ms) are both
//! significantly faster than creating fresh browsers (~1050ms per call).
//!
//! ## Performance Analysis Summary
//!
//! We tested three approaches and prioritized for overall speed over
//! consistency:
//!
//! 1. **Fresh instances per call**: Consistently ~1050ms
//! 2. **Complex browser pooling**: 238-337ms first (after warmup), ~400ms
//!    average - pooling overhead made subsequent calls slower
//! 3. **Static browser/tab reuse**: 288-298ms first (after warmup), 379-386ms
//!    subsequent - optimal speed with simple architecture
//!
//! This implementation prioritizes speed for the first 1-3 calls, which are
//! most critical for user experience. Use `warmup()` during application startup
//! for optimal first-call performance.
//!
//! Key performance optimizations:
//!
//! - Optimized Chrome launch arguments
//! - Direct content injection via `Page::SetDocumentContent`
//! - Optimized screenshot parameters (`quality: 5`, `optimize_for_speed: true`)
//! - Pre-warmed browser instance with tab ready to use

use std::{
    ffi::OsStr,
    sync::{Arc, Mutex},
};

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use headless_chrome::{Browser, LaunchOptionsBuilder, Tab, protocol::cdp::Page};

use common::{
    eyre::{Result, eyre},
    itertools::Itertools,
    once_cell::sync::Lazy,
};

/// Converts HTML to PNG and returns as data URI
///
/// This function uses a persistent browser instance for optimal performance.
/// Optionally, call `warmup()` during application startup for optimal first-call performance.
///
/// # Arguments
/// * `html`: HTML content to render
///
/// # Returns
/// * `Result<String>`: Base64 encoded PNG as a data URI
///
/// # Example
/// ```no_run
/// use convert::html_to_png::{self, warmup};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // Warmup for optimal first-call performance
/// html_to_png::warmup()?;
///
/// let html = "<table><tr><td>Hello</td></tr></table>";
/// let data_uri = html_to_png(html)?;
/// // Returns: "data:image/png;base64,iVBORw0KGgoAAAANS..."
/// # Ok(())
/// # }
/// ```
pub fn html_to_png(html: &str) -> Result<String> {
    let png_data = capture_screenshot(html)?;

    // Encode PNG data as base64 for data URI
    let base64_png = BASE64.encode(&png_data);

    // Return as data URI
    Ok(format!("data:image/png;base64,{}", base64_png))
}

/// Static browser and tab instances that are reused across function calls
static BROWSER: Lazy<Mutex<Option<Browser>>> = Lazy::new(|| Mutex::new(None));
static TAB: Lazy<Mutex<Option<Arc<Tab>>>> = Lazy::new(|| Mutex::new(None));

/// Warm up the browser by creating initial instance
///
/// Call this during application startup to avoid cold start delays
/// on the first screenshot request.
pub fn warmup() -> Result<()> {
    ensure_browser_available()
}

/// Shutdown the browser and clean up resources
///
/// Call this during application shutdown for clean resource cleanup.
pub fn shutdown() -> Result<()> {
    if let Ok(mut browser_guard) = BROWSER.lock() {
        *browser_guard = None;
    }
    if let Ok(mut tab_guard) = TAB.lock() {
        *tab_guard = None;
    }

    Ok(())
}

/// Creates a new browser instance with optimized launch options
fn create_browser() -> Result<Browser> {
    // Optimized Chrome launch arguments for HTML to PNG conversion
    let args: Vec<&OsStr> = [
        "--disable-dev-shm-usage",          // Memory optimization for containers
        "--disable-extensions",             // No extensions needed for headless rendering
        "--disable-gpu",                    // Disable GPU for headless stability
        "--disable-hang-monitor",           // Disable hang monitoring for performance
        "--disable-renderer-backgrounding", // Prevent background throttling
        "--force-device-scale-factor=1",    // Ensure consistent output scaling
        "--memory-pressure-off",            // Disable memory pressure monitoring
        "--run-all-compositor-stages-before-draw", // Rendering optimization
    ]
    .iter()
    .map(OsStr::new)
    .collect_vec();

    let options = LaunchOptionsBuilder::default()
        .headless(true)
        .args(args)
        .build()
        .map_err(|e| eyre!("Failed to build browser launch options: {}", e))?;

    Browser::new(options).map_err(|e| eyre!("Failed to create browser instance: {}", e))
}

/// Ensures we have a working browser and tab instance, recreating if necessary
fn ensure_browser_available() -> Result<()> {
    let mut browser_guard = BROWSER
        .lock()
        .map_err(|e| eyre!("Failed to acquire browser lock: {}", e))?;

    let mut tab_guard = TAB
        .lock()
        .map_err(|e| eyre!("Failed to acquire tab lock: {}", e))?;

    // If we already have both browser and tab, assume they're working
    if browser_guard.is_some() && tab_guard.is_some() {
        return Ok(());
    }

    // Create a new browser instance
    let new_browser = create_browser()?;
    let new_tab = new_browser
        .new_tab()
        .map_err(|e| eyre!("Failed to create initial tab: {}", e))?;

    // Pre-warm the tab with a minimal document (for optimal first-call performance)
    let frame_tree = new_tab
        .call_method(Page::GetFrameTree(None))
        .map_err(|e| eyre!("Failed to get frame tree for warmup: {}", e))?;

    new_tab
        .call_method(Page::SetDocumentContent {
            frame_id: frame_tree.frame_tree.frame.id,
            html: "<html><body></body></html>".to_string(),
        })
        .map_err(|e| eyre!("Failed to warm up tab: {}", e))?;

    *browser_guard = Some(new_browser);
    *tab_guard = Some(new_tab);

    Ok(())
}

/// Captures HTML content as PNG using persistent browser instance
fn capture_screenshot(html: &str) -> Result<Vec<u8>> {
    // Ensure we have a working browser, recreating if necessary
    ensure_browser_available()?;

    let result = try_screenshot(html);

    // If screenshot failed, try recreating the browser and retry once
    if result.is_err() {
        // Force recreation by clearing both browser and tab
        if let Ok(mut browser_guard) = BROWSER.lock() {
            *browser_guard = None;
        }
        if let Ok(mut tab_guard) = TAB.lock() {
            *tab_guard = None;
        }

        // Ensure browser and tab are available again
        ensure_browser_available()?;

        // Retry the screenshot
        try_screenshot(html)
    } else {
        result
    }
}

/// Attempts to take a screenshot with the current tab instance
fn try_screenshot(html: &str) -> Result<Vec<u8>> {
    let tab_guard = TAB
        .lock()
        .map_err(|e| eyre!("Failed to acquire tab lock: {}", e))?;

    let tab = tab_guard
        .as_ref()
        .ok_or_else(|| eyre!("No tab instance available"))?;

    // Use Page.setDocumentContent for fastest content injection
    let frame_tree = tab
        .call_method(Page::GetFrameTree(None))
        .map_err(|e| eyre!("Failed to get frame tree: {}", e))?;

    tab.call_method(Page::SetDocumentContent {
        frame_id: frame_tree.frame_tree.frame.id,
        html: html.to_string(),
    })
    .map_err(|e| eyre!("Failed to set document content: {}", e))?;

    // Skip DOM ready check - Page.setDocumentContent is synchronous

    // Capture screenshot with maximum speed optimization
    let screenshot_params = Page::CaptureScreenshot {
        format: Some(Page::CaptureScreenshotFormatOption::Png),
        quality: Some(5), // Lower quality for speed
        clip: None,       // Full page
        from_surface: Some(true),
        capture_beyond_viewport: Some(false), // Don't capture beyond viewport for speed
        optimize_for_speed: Some(true),
    };

    let result = tab
        .call_method(screenshot_params)
        .map_err(|e| eyre!("Failed to capture screenshot: {}", e))?;

    // Decode the base64 screenshot data
    let png_data = BASE64
        .decode(&result.data)
        .map_err(|e| eyre!("Failed to decode screenshot data: {}", e))?;

    Ok(png_data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    #[test]
    fn test_simple_table() -> Result<()> {
        let html = r#"
            <table border="1">
                <tr><th>Name</th><th>Value</th></tr>
                <tr><td>Test</td><td>123</td></tr>
            </table>
        "#;

        let data_uri = html_to_png(html)?;
        assert!(data_uri.starts_with("data:image/png;base64,"));

        Ok(())
    }

    #[test]
    fn test_perf() -> Result<()> {
        // Ensure clean state
        shutdown()?;

        let html = "<table><tr><td>Performance Test</td></tr></table>";

        // First capture should be slower because browser needs to be instantiated
        let start = Instant::now();
        let _result = html_to_png(html)?;
        let first_duration = start.elapsed();

        // Subsequent captures should be faster because they reuse the browser
        let mut total_duration = Duration::ZERO;
        for _ in 0..10 {
            let start = Instant::now();
            let _result = html_to_png(html)?;
            total_duration += start.elapsed();
        }
        let avg_duration = total_duration / 10;

        dbg!(first_duration);
        dbg!(avg_duration);

        assert!(
            avg_duration.as_millis() < first_duration.as_millis(),
            "Average duration should be less than first got {avg_duration:?} > {first_duration:?}",
        );

        Ok(())
    }
}
