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
    time::Duration,
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
    tokio, tracing,
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
    html_to_png_data_uri_with_padding(html, 16)
}

/// Converts HTML to PNG and returns as data URI with configurable padding
///
/// This function uses a persistent browser instance for optimal performance and
/// crops the screenshot to the content bounds with the specified padding.
///
/// # Arguments
/// * `html`: HTML content to render
/// * `padding`: Padding in pixels around the content (0 for tight cropping)
///
/// # Returns
/// * `Result<String>`: Base64 encoded PNG as a data URI
pub fn html_to_png_data_uri_with_padding(html: &str, padding: u32) -> Result<String> {
    let base64_png = capture_screenshot_with_padding(&wrap_html(html), padding)?;

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
    html_to_png_file_with_padding(html, path, 16)
}

/// Converts HTML to PNG and saves to file with configurable padding
///
/// This function uses a persistent browser instance for optimal performance and
/// crops the screenshot to the content bounds with the specified padding.
///
/// # Arguments
/// * `html`: HTML content to render
/// * `path`: File path where the PNG will be saved
/// * `padding`: Padding in pixels around the content (0 for tight cropping)
///
/// # Returns
/// * `Result<()>`: Success or error
pub fn html_to_png_file_with_padding(html: &str, path: &Path, padding: u32) -> Result<()> {
    let base64_png = capture_screenshot_with_padding(&wrap_html(html), padding)?;

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

/// Media type timeout configurations for dynamic rendering
struct MediaTypeTimeout {
    media_type: &'static str,
    wait_ms: u32,
    timeout_ms: u32,
}

const MEDIA_TYPE_TIMEOUTS: &[MediaTypeTimeout] = &[
    MediaTypeTimeout {
        media_type: "application/vnd.plotly.v1+json",
        wait_ms: 0,
        timeout_ms: 3000,
    },
    MediaTypeTimeout {
        media_type: "text/vnd.mermaid",
        wait_ms: 1000,
        timeout_ms: 8000,
    },
    MediaTypeTimeout {
        media_type: "application/vnd.vegalite.v5+json",
        wait_ms: 0,
        timeout_ms: 3000,
    },
    MediaTypeTimeout {
        media_type: "application/vnd.vega.v5+json",
        wait_ms: 0,
        timeout_ms: 3000,
    },
    MediaTypeTimeout {
        media_type: "application/vnd.cytoscape.v3+json",
        wait_ms: 0,
        timeout_ms: 2000,
    },
    MediaTypeTimeout {
        media_type: "text/html", // Leaflet maps
        wait_ms: 0,
        timeout_ms: 4000,
    },
];

/// Check if HTML contains stencila-image-object elements that require JavaScript for rendering
fn needs_dynamic_scripts(html: &str) -> bool {
    // Check if HTML contains stencila-image-object with any of the dynamic media types
    html.contains("<stencila-image-object")
        && MEDIA_TYPE_TIMEOUTS
            .iter()
            .any(|timeout| html.contains(timeout.media_type))
}

/// Detect which media types are present in the HTML and return their timeouts
fn get_media_type_timeouts(html: &str) -> Vec<&'static MediaTypeTimeout> {
    MEDIA_TYPE_TIMEOUTS
        .iter()
        .filter(|timeout| html.contains(timeout.media_type))
        .collect()
}

/// Wait for rendering completion based on media type detection
async fn detect_rendering_completion(tab: &Arc<Tab>, html: &str) -> Result<()> {
    // If no dynamic scripts are needed, return immediately
    if !needs_dynamic_scripts(html) {
        tracing::debug!("No dynamic content detected, skipping render wait");
        return Ok(());
    }

    let media_timeouts = get_media_type_timeouts(html);
    if media_timeouts.is_empty() {
        tracing::debug!("No matching media types found");
        return Ok(());
    }

    tracing::debug!(
        "Detected {} dynamic media type(s), waiting for completion",
        media_timeouts.len()
    );

    // Wait for web components to be defined first
    let web_component_check = r#"
        return new Promise((resolve) => {
            if (customElements.get('stencila-image-object')) {
                resolve(true);
            } else {
                customElements.whenDefined('stencila-image-object').then(() => resolve(true));
            }
        });
    "#;
    if let Err(error) = tab.evaluate(web_component_check, true) {
        tracing::warn!("Failed to wait for stencila-image-object to be defined: {error}",);
    } else {
        tracing::debug!("stencila-image-object web component is defined");
    }

    // Initial wait for modules and component initialization
    let max_wait = media_timeouts
        .iter()
        .map(|t| t.wait_ms)
        .max()
        .unwrap_or(1000);
    tokio::time::sleep(Duration::from_millis(max_wait as u64)).await;

    // Use JavaScript to check completion by penetrating shadow DOM
    let rendering_comple_check = r#"
        return new Promise((resolve) => {
            let checkCount = 0;
            const maxChecks = 30; // 15 seconds max (500ms intervals)
            
            function checkCompletion() {
                checkCount++;
                
                const imageObjects = document.querySelectorAll('stencila-image-object');
                let allComplete = true;
                
                imageObjects.forEach((element) => {
                    const mediaType = element.getAttribute('media-type');
                    
                    if (element.shadowRoot) {
                        const shadowContent = element.shadowRoot;
                        
                        if (mediaType === 'text/vnd.mermaid') {
                            const svgs = shadowContent.querySelectorAll('svg');
                            if (svgs.length === 0) {
                                allComplete = false;
                            }
                        } else if (mediaType === 'application/vnd.plotly.v1+json') {
                            const plots = shadowContent.querySelectorAll('.plotly');
                            if (plots.length === 0) {
                                allComplete = false;
                            }
                        } else if (mediaType && mediaType.includes('vega')) {
                            const vegaElements = shadowContent.querySelectorAll('.vega-embed');
                            if (vegaElements.length === 0) {
                                allComplete = false;
                            }
                        }
                    } else {
                        allComplete = false;
                    }
                });
                
                if (allComplete || checkCount >= maxChecks) {
                    resolve(allComplete);
                } else {
                    setTimeout(checkCompletion, 500);
                }
            }
            
            checkCompletion();
        });
    "#;
    match tab.evaluate(rendering_comple_check, true) {
        Ok(_) => {
            tracing::debug!("Rendering completion check succeeded");
        }
        Err(error) => {
            tracing::warn!("Rendering completion check failed: {error}. Using fallback delay.",);
            // Fallback to a generous delay for complex visualizations
            let max_timeout = media_timeouts
                .iter()
                .map(|t| t.timeout_ms)
                .max()
                .unwrap_or(3000);
            tokio::time::sleep(Duration::from_millis(max_timeout as u64)).await;
        }
    }

    Ok(())
}

/// Calculate the bounding box of the content wrapper for cropping
async fn get_content_bounds(tab: &Arc<Tab>, padding: u32) -> Result<Option<Page::Viewport>> {
    let bounds_js = format!(
        r#"
        new Promise((resolve) => {{
            const wrapper = document.getElementById('stencila-content-wrapper');
            if (!wrapper) {{
                resolve(null);
                return;
            }}
            
            const rect = wrapper.getBoundingClientRect();
            const padding = {padding};
            
            // Ensure we have meaningful content dimensions
            if (rect.width < 1 || rect.height < 1) {{
                resolve(null);
                return;
            }}
            
            // Calculate clipping bounds with padding
            const bounds = {{
                x: Math.max(0, rect.left - padding),
                y: Math.max(0, rect.top - padding), 
                width: rect.width + (2 * padding),
                height: rect.height + (2 * padding),
                scale: 1
            }};
            
            resolve(bounds);
        }})
    "#
    );

    let result = match tab.evaluate(&bounds_js, true) {
        Ok(result) => result,
        Err(error) => {
            tracing::warn!("Failed to calculate content bounds: {error}");
            return Ok(None);
        }
    };

    // Try to get the actual object properties using headless_chrome's get_properties
    if let Some(object_id) = &result.object_id {
        match tab.call_method(headless_chrome::protocol::cdp::Runtime::GetProperties {
            object_id: object_id.clone(),
            own_properties: Some(true),
            accessor_properties_only: Some(false),
            generate_preview: Some(false),
            non_indexed_properties_only: Some(false),
        }) {
            Ok(props_result) => {
                let mut x = 0.0;
                let mut y = 0.0;
                let mut width = 0.0;
                let mut height = 0.0;
                let mut scale = 1.0;

                for prop in props_result.result {
                    let Some(value) = &prop.value else { continue };
                    let value = value.value.as_ref().and_then(|v| v.as_f64());
                    match prop.name.as_str() {
                        "x" => x = value.unwrap_or(0.0),
                        "y" => y = value.unwrap_or(0.0),
                        "width" => width = value.unwrap_or(0.0),
                        "height" => height = value.unwrap_or(0.0),
                        "scale" => scale = value.unwrap_or(1.0),
                        _ => {}
                    }
                }

                if width > 0.0 && height > 0.0 {
                    let viewport = Page::Viewport {
                        x,
                        y,
                        width,
                        height,
                        scale,
                    };
                    tracing::debug!("Content bounds detected: {width}×{height} at ({x}, {y})",);
                    return Ok(Some(viewport));
                }
            }
            Err(error) => {
                tracing::warn!("Failed to get object properties: {error}");
            }
        }
    }

    tracing::debug!("Could not detect content bounds, using full page screenshot");
    Ok(None)
}

/// Wraps HTML with so that any necessary CSS and Javascript is available
fn wrap_html(html: &str) -> String {
    // Keep any unused linting warning here so that if we turn on the line below
    // it is harder to forget to reverse that before committing
    let _static_prefix = format!("https://stencila.io/web/v{STENCILA_VERSION}");

    // During development of web components uncomment the next line and run
    // a local Stencila server with permissive CORS (so that headless
    // browser can get use web dist):
    //
    // cargo run -p cli serve --cors permissive
    let static_prefix = "http://localhost:9000/~static/dev".to_string();

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
        <style>
            body {{
                margin: 0;
                padding: 0;
                font-family: system-ui, -apple-system, sans-serif;
            }}
            #stencila-content-wrapper {{
                display: inline-block;
                min-width: fit-content;
                min-height: fit-content;
            }}
        </style>
        {scripts}
    </head>
    <body>
        <div id="stencila-content-wrapper">
            <stencila-dynamic-view view=dynamic>
                {html}
            </stencila-dynamic-view>
        </div>
    </body>
</html>"#
    )
}

/// Captures HTML content as PNG using persistent browser instance with content cropping
fn capture_screenshot_with_padding(html: &str, padding: u32) -> Result<String> {
    // Testing indicated that for Plotly is is best to use a fresh browser tab
    if html.contains("application/vnd.plotly.v1+json") {
        shutdown().ok();
    }

    // Ensure we have a working browser, recreating if necessary
    ensure_browser_available()?;

    let result = try_screenshot_with_padding(html, padding);
    if result.is_err() {
        // Force recreation by clearing both browser and tab
        if let Ok(mut manager) = BROWSER_MANAGER.lock() {
            manager.clear_browser_and_tab();
        }

        // Ensure browser and tab are available again
        ensure_browser_available()?;

        // Retry the screenshot
        try_screenshot_with_padding(html, padding)
    } else {
        result
    }
}

/// Attempts to take a screenshot with the current tab instance, cropped to content
fn try_screenshot_with_padding(html: &str, padding: u32) -> Result<String> {
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
    tab.add_event_listener(Arc::new(move |event: &Event| {
        if let Event::LogEntryAdded(entry) = event {
            tracing::debug!("{:?} {}", entry.params.entry.level, entry.params.entry.text)
        }
    }))
    .map_err(|error| eyre!("Failed to add log event listener: {error}"))?;

    // Wait for interactive visualizations to complete rendering
    // This detects media types and waits only when necessary
    tokio::task::block_in_place(|| match tokio::runtime::Handle::try_current() {
        Ok(handle) => {
            if let Err(error) = handle.block_on(detect_rendering_completion(tab, html)) {
                tracing::warn!(
                    "Rendering completion detection failed: {error}. Taking screenshot anyway.",
                );
            }
        }
        Err(_) => {
            tracing::debug!("No tokio runtime available, skipping async completion detection");
        }
    });

    // Calculate content bounds for cropping
    let clip = tokio::task::block_in_place(|| match tokio::runtime::Handle::try_current() {
        Ok(handle) => match handle.block_on(get_content_bounds(tab, padding)) {
            Ok(bounds) => bounds,
            Err(error) => {
                tracing::warn!(
                    "Failed to get content bounds: {error}. Using full page screenshot."
                );
                None
            }
        },
        Err(_) => {
            tracing::debug!("No tokio runtime available, using full page screenshot");
            None
        }
    });

    // Capture screenshot with content cropping or full page fallback
    let has_clip = clip.is_some();
    let screenshot_params = Page::CaptureScreenshot {
        format: Some(Page::CaptureScreenshotFormatOption::Png),
        quality: Some(5), // Lower quality for speed
        clip,             // Use content bounds or None for full page
        from_surface: Some(true),
        capture_beyond_viewport: Some(false), // Don't capture beyond viewport for speed
        optimize_for_speed: Some(true),
    };

    let result = tab
        .call_method(screenshot_params)
        .map_err(|error| eyre!("Failed to capture screenshot: {error}"))?;

    if has_clip {
        tracing::debug!("Screenshot captured with content cropping");
    } else {
        tracing::debug!("Screenshot captured without cropping (fallback to full page)");
    }

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

    /// To run this test with logs printed:
    /// RUST_LOG=trace cargo test -p convert html_to_png::tests::test_rendering -- --nocapture
    #[test_log::test(tokio::test(flavor = "multi_thread", worker_threads = 2))]
    async fn test_rendering() -> Result<()> {
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

            let wrapped_html = wrap_html(html);
            html_to_png_file(&wrapped_html, &output_path)?;

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
