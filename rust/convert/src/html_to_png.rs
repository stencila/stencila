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
    path::Path,
    sync::{Arc, Mutex},
};

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use headless_chrome::{
    Browser, LaunchOptionsBuilder, Tab,
    protocol::cdp::{Page, types::Event},
};

use common::{
    eyre::{Result, eyre},
    itertools::Itertools,
    once_cell::sync::Lazy,
    tracing,
};
use version::STENCILA_VERSION;
use web_dist::Web;

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
/// let data_uri = html_to_png_data_uri(html)?;
/// // Returns: "data:image/png;base64,iVBORw0KGgoAAAANS..."
///
/// // With Stencila assets for rendering components like Mermaid, Plotly
/// let data_uri = html_to_png_data_uri(html)?;
/// # Ok(())
/// # }
/// ```
pub fn html_to_png_data_uri(html: &str) -> Result<String> {
    let base64_png = capture_screenshot(&wrap_html(html))?;

    // Return as data URI (base64 string already from Chrome)
    Ok(format!("data:image/png;base64,{}", base64_png))
}

/// Converts HTML to PNG and saves to file
///
/// This function uses a persistent browser instance for optimal performance.
/// Optionally, call `warmup()` during application startup for optimal first-call performance.
///
/// # Arguments
/// * `html`: HTML content to render
/// * `path`: File path where the PNG will be saved
///
/// # Returns
/// * `Result<()>`: Success or error
///
/// # Example
/// ```no_run
/// use convert::html_to_png::{html_to_png_file, warmup};
/// use std::path::Path;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // Warmup for optimal first-call performance
/// warmup()?;
///
/// let html = "<table><tr><td>Hello</td></tr></table>";
/// html_to_png_file(html, Path::new("output.png"))?;
///
/// # Ok(())
/// # }
/// ```
pub fn html_to_png_file(html: &str, path: &Path) -> Result<()> {
    let base64_png = capture_screenshot(&wrap_html(html))?;

    // Decode base64 to bytes for file writing
    let png_bytes = BASE64
        .decode(&base64_png)
        .map_err(|error| eyre!("Failed to decode base64 PNG data: {error}"))?;

    // Write to file
    std::fs::write(path, &png_bytes)
        .map_err(|error| eyre!("Failed to write PNG file to {}: {error}", path.display()))?;

    Ok(())
}

/// Browser manager that automatically cleans up resources when dropped
/// 
/// Avoids zombie chrome processes if shutdown is not explicitly called.
struct BrowserManager {
    browser: Option<Browser>,
    tab: Option<Arc<Tab>>,
}

impl BrowserManager {
    fn new() -> Self {
        Self {
            browser: None,
            tab: None,
        }
    }

    fn has_browser_and_tab(&self) -> bool {
        self.browser.is_some() && self.tab.is_some()
    }

    fn set_browser_and_tab(&mut self, browser: Browser, tab: Arc<Tab>) {
        self.browser = Some(browser);
        self.tab = Some(tab);
    }

    fn clear_browser_and_tab(&mut self) {
        self.browser = None;
        self.tab = None;
    }

    fn tab(&self) -> Option<&Arc<Tab>> {
        self.tab.as_ref()
    }

    /// Manual cleanup method that can be called explicitly
    fn cleanup(&mut self) {
        // Clean up browser and tab
        if self.browser.is_some() || self.tab.is_some() {
            tracing::debug!("Cleaning up browser and tab instances");
            self.browser = None;
            self.tab = None;
        }
    }
}

impl Drop for BrowserManager {
    fn drop(&mut self) {
        tracing::debug!("BrowserManager being dropped, cleaning up resources");
        self.cleanup();
    }
}

/// Static browser manager instance that is reused across function calls
static BROWSER_MANAGER: Lazy<Mutex<BrowserManager>> =
    Lazy::new(|| Mutex::new(BrowserManager::new()));

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
/// 
/// Note: Resources will be automatically cleaned up when the program exits
/// even if this function is not called, thanks to the Drop implementation.
pub fn shutdown() -> Result<()> {
    if let Ok(mut manager) = BROWSER_MANAGER.lock() {
        manager.cleanup();
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
        // During development, it is very useful to set headless: false to be able
        // to inspect generated HTML and any JS errors in the console
        .headless(true)
        .args(args)
        .build()
        .map_err(|error| eyre!("Failed to build browser launch options: {error}"))?;

    Browser::new(options).map_err(|error| eyre!("Failed to create browser instance: {error}"))
}

/// Ensures we have a working browser and tab instance, recreating if necessary
fn ensure_browser_available() -> Result<()> {
    let mut manager = BROWSER_MANAGER
        .lock()
        .map_err(|error| eyre!("Failed to acquire browser manager lock: {error}"))?;

    // If we already have both browser and tab, assume they're working
    if manager.has_browser_and_tab() {
        return Ok(());
    }

    // Create a new browser instance
    let new_browser = create_browser()?;
    let new_tab = new_browser
        .new_tab()
        .map_err(|error| eyre!("Failed to create initial tab: {error}"))?;

    // Pre-warm the tab with a minimal document (for optimal first-call performance)
    let frame_tree = new_tab
        .call_method(Page::GetFrameTree(None))
        .map_err(|error| eyre!("Failed to get frame tree for warmup: {error}"))?;

    new_tab
        .call_method(Page::SetDocumentContent {
            frame_id: frame_tree.frame_tree.frame.id,
            html: "<html><body></body></html>".to_string(),
        })
        .map_err(|error| eyre!("Failed to warm up tab: {error}"))?;

    manager.set_browser_and_tab(new_browser, new_tab);

    Ok(())
}

/// Check if HTML contains stencila-image-object elements that require JavaScript for rendering
fn needs_dynamic_scripts(html: &str) -> bool {
    // Media types that require JavaScript modules for proper rendering
    // See `web/src/nodes/image-object.ts`
    const DYNAMIC_MEDIA_TYPES: &[&str] = &[
        "application/vnd.cytoscape.v3+json",
        "application/vnd.plotly.v1+json",
        "application/vnd.vega.v5+json",
        "application/vnd.vegalite.v5+json",
        "text/html", // Leaflet maps
        "text/vnd.mermaid",
    ];

    // Check if HTML contains stencila-image-object with any of the dynamic media types
    html.contains("<stencila-image-object")
        && DYNAMIC_MEDIA_TYPES
            .iter()
            .any(|media_type| html.contains(media_type))
}

/// Wraps HTML with so that any necessary CSS and Javascript is available
fn wrap_html(html: &str) -> String {
    // Keep any unused linting warning here so that if we turn on the line below
    // it is harder to forget to reverse that before committing
    let static_prefix = format!("https://stencila.io/web/v{STENCILA_VERSION}");

    // During development of web components uncomment the next line and run
    // a local Stencila server with permissive CORS (so that headless
    // browser can get use web dist):
    //
    // cargo run -p cli serve --cors permissive
    let static_prefix = format!("http://localhost:9000/~static/dev");

    // Add CSS
    let mut styles = String::new();
    for path in ["themes/default.css", "views/dynamic.css"] {
        if let Some(file) = Web::get(path) {
            // Inject CSS directly
            let content = String::from_utf8_lossy(&file.data);
            styles.push_str(&format!(r#"<style>{content}</style>"#));
        } else {
            // Fallback to link to CSS
            styles.push_str(&format!(
                r#"<link rel="stylesheet" type="text/css" href="{static_prefix}/{path}">"#
            ));
        }
    }

    // Add JavaScript if necessary - only when HTML contains interactive visualizations
    // that require dynamic module loading (Plotly, Mermaid, Vega-Lite, etc.)
    // Due to the way that modules are dynamically, asynchronously loaded it is not
    // possible to inject these, they must come from a server.
    let scripts = if needs_dynamic_scripts(html) {
        format!(r#"<script type="module" src="{static_prefix}/views/dynamic.js"></script>"#)
    } else {
        String::new()
    };

    format!(
        r#"<!doctype html>
<html lang="en">
    <head>
        <meta charset="utf-8"/>
        <title>Stencila Screen</title>
        <meta name="viewport" content="width=device-width, initial-scale=1" />
        {styles}
        {scripts}
    </head>
    <body>
        <stencila-dynamic-view view=dynamic>
            {html}
        </stencila-dynamic-view>
    </body>
</html>"#
    )
}

/// Captures HTML content as PNG using persistent browser instance
fn capture_screenshot(html: &str) -> Result<String> {
    // Ensure we have a working browser, recreating if necessary
    ensure_browser_available()?;

    let result = try_screenshot(&html);

    // If screenshot failed, try recreating the browser and retry once
    if result.is_err() {
        // Force recreation by clearing both browser and tab
        if let Ok(mut manager) = BROWSER_MANAGER.lock() {
            manager.clear_browser_and_tab();
        }

        // Ensure browser and tab are available again
        ensure_browser_available()?;

        // Retry the screenshot
        try_screenshot(&html)
    } else {
        result
    }
}

/// Attempts to take a screenshot with the current tab instance
fn try_screenshot(html: &str) -> Result<String> {
    let manager = BROWSER_MANAGER
        .lock()
        .map_err(|error| eyre!("Failed to acquire browser manager lock: {error}"))?;

    let tab = manager
        .tab()
        .ok_or_else(|| eyre!("No tab instance available"))?;

    // Use Page.setDocumentContent for fastest content injection
    let frame_tree = tab
        .call_method(Page::GetFrameTree(None))
        .map_err(|error| eyre!("Failed to get frame tree: {error}"))?;

    tab.call_method(Page::SetDocumentContent {
        frame_id: frame_tree.frame_tree.frame.id,
        html: html.to_string(),
    })
    .map_err(|error| eyre!("Failed to set document content: {error}"))?;

    // Set global variable to enable static mode for interactive visualizations
    tab.evaluate("window.STENCILA_STATIC_MODE = true;", false)
        .map_err(|error| eyre!("Failed to set static mode: {error}"))?;

    tab.enable_log()
        .map_err(|error| eyre!("Failed to enable log: {error}"))?;
    tab.add_event_listener(Arc::new(move |event: &Event| match event {
        Event::LogEntryAdded(entry) => {
            tracing::debug!("{:?} {}", entry.params.entry.level, entry.params.entry.text)
        }
        _ => {}
    }))
    .map_err(|error| eyre!("Failed to add log event listener: {error}"))?;

    // TODO: implement a way for interactive visualizations to (a) signal that they
    // are rendering and (b) finished rendering so that this can wait but only if needed.
    //tab.wait_for_element_with_custom_timeout("stencila-image-object > svg", Duration::from_millis(5000))
    //    .map_err(|error| eyre!(error))?;

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
        .map_err(|error| eyre!("Failed to capture screenshot: {error}"))?;

    Ok(result.data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    #[test]
    fn test_browser_manager_drop() {
        // This test verifies that BrowserManager properly implements Drop
        // By creating a scope where a BrowserManager is created and dropped
        {
            let mut manager = BrowserManager::new();
            // The manager should be empty initially
            assert!(!manager.has_browser_and_tab());

            // Test manual cleanup
            manager.cleanup();
            assert!(!manager.has_browser_and_tab());
        }
        // When the manager goes out of scope, Drop::drop should be called
        // This is automatically tested by the compiler/runtime
    }

    #[test]
    fn test_needs_dynamic_scripts() {
        // HTML without stencila-image-object should not need scripts
        let simple_html = "<p>Hello world</p>";
        assert!(!needs_dynamic_scripts(simple_html));

        // HTML with stencila-image-object but no dynamic media types should not need scripts
        let static_html =
            r#"<stencila-image-object media-type="image/png">test</stencila-image-object>"#;
        assert!(!needs_dynamic_scripts(static_html));

        // HTML with Mermaid should need scripts
        let mermaid_html =
            r#"<stencila-image-object media-type="text/vnd.mermaid">test</stencila-image-object>"#;
        assert!(needs_dynamic_scripts(mermaid_html));

        // HTML with Plotly should need scripts
        let plotly_html = r#"<stencila-image-object media-type="application/vnd.plotly.v1+json">test</stencila-image-object>"#;
        assert!(needs_dynamic_scripts(plotly_html));

        // HTML with Vega-Lite should need scripts
        let vega_html = r#"<stencila-image-object media-type="application/vnd.vegalite.v5+json">test</stencila-image-object>"#;
        assert!(needs_dynamic_scripts(vega_html));
    }

    #[test]
    fn test_perf() -> Result<()> {
        // Ensure clean state
        shutdown()?;

        let html = "<table><tr><td>Performance Test</td></tr></table>";

        // First capture should be slower because browser needs to be instantiated
        let start = Instant::now();
        let _result = html_to_png_data_uri(html)?;
        let first_duration = start.elapsed();

        // Subsequent captures should be faster because they reuse the browser
        let mut total_duration = Duration::ZERO;
        for _ in 0..10 {
            let start = Instant::now();
            let _result = html_to_png_data_uri(html)?;
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

    // This test will normally be ignored
    #[test]
    fn test_rendering() -> Result<()> {
        // Test Mermaid rendering
        let mermaid_html = r#"
            <stencila-image-object 
                media-type="text/vnd.mermaid" 
                content-url="graph TD
    A --> B
">
            </stencila-image-object>
        "#;

        // Test Plotly rendering
        let plotly_html = r#"
            <stencila-image-object 
                media-type="application/vnd.plotly.v1+json" 
                content-url='{
                    "data": [{
                        "x": ["Jan", "Feb", "Mar", "Apr", "May"],
                        "y": [20, 14, 23, 25, 22],
                        "type": "scatter",
                        "mode": "lines+markers",
                        "name": "Sales"
                    }],
                    "layout": {
                        "title": "Monthly Sales Data",
                        "xaxis": {"title": "Month"},
                        "yaxis": {"title": "Sales ($k)"}
                    }
                }'>
            </stencila-image-object>
        "#;

        // Test Vega-Lite rendering
        let vega_html = r#"
            <stencila-image-object 
                media-type="application/vnd.vegalite.v5+json" 
                content-url='{
                    "$schema": "https://vega.github.io/schema/vega-lite/v5.json",
                    "description": "A simple bar chart with embedded data.",
                    "data": {
                        "values": [
                            {"category": "A", "value": 28},
                            {"category": "B", "value": 55},
                            {"category": "C", "value": 43},
                            {"category": "D", "value": 91},
                            {"category": "E", "value": 81}
                        ]
                    },
                    "mark": "bar",
                    "encoding": {
                        "x": {"field": "category", "type": "nominal", "axis": {"labelAngle": 0}},
                        "y": {"field": "value", "type": "quantitative"}
                    },
                    "title": "Sample Bar Chart"
                }'>
            </stencila-image-object>
        "#;

        // Render all examples
        let examples = [
            ("test-mermaid.png", mermaid_html),
            ("test-plotly.png", plotly_html),
            ("test-vega-lite.png", vega_html),
        ];

        for (filename, html) in examples {
            let output_path = Path::new(filename);

            if output_path.exists() {
                std::fs::remove_file(&output_path)?;
            }

            html_to_png_file(&wrap_html(html), &output_path)?;

            let file_size = std::fs::metadata(&output_path)?.len();
            assert!(
                file_size > 1000,
                "PNG file should have substantial content (got {} bytes)",
                file_size
            );
        }

        Ok(())
    }
}
