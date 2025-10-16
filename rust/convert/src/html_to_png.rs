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
    sync::{Arc, LazyLock, Mutex},
    thread::sleep,
    time::{Duration, Instant},
};

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use eyre::{Result, bail, eyre};
use headless_chrome::{
    Browser, LaunchOptionsBuilder, Tab,
    protocol::cdp::{Page, Runtime, types::Event},
    types::PrintToPdfOptions,
};
use itertools::Itertools;

use stencila_themes::LengthConversion;
use stencila_tools::{install_sync, is_installable, is_installed};
use stencila_version::STENCILA_VERSION;
use stencila_web_dist::Web;

/// Duration in seconds to keep browser open for inspection during development.
/// Set to 0 for normal operation (headless mode).
/// Set to > 0 to keep browser window open for debugging.
#[cfg(debug_assertions)]
const BROWSER_OPEN_SECS: u64 = 0;

/// Use local development web assets instead of production CDN.
/// Set to false for normal operation (uses production CDN).
/// Set to true for local development (requires running `cargo run --bin stencila serve --cors permissive`).
#[cfg(debug_assertions)]
const USE_LOCALHOST: bool = false;

/// Converts HTML to PNG and returns as data URI
///
/// # Arguments
/// * `html`: HTML content to render
///
/// # Returns
/// * `Result<String>`: Base64 encoded PNG as a data URI
pub fn html_to_png_data_uri(html: &str) -> Result<String> {
    html_to_png_data_uri_with(html, 16, ConsoleErrorHandling::FailOnErrors)
}

/// Converts HTML to PNG and returns as data URI with configurable padding and error handling
///
/// # Arguments
/// * `html`: HTML content to render
/// * `padding`: Padding in pixels around the content (0 for tight cropping)
/// * `console_error_handling`: How to handle JavaScript console errors
///
/// # Returns
/// * `Result<String>`: Base64 encoded PNG as a data URI
pub fn html_to_png_data_uri_with(
    html: &str,
    padding: u32,
    console_error_handling: ConsoleErrorHandling,
) -> Result<String> {
    let base64_png = capture_png(&wrap_html(html), padding, console_error_handling)?;

    // Return as data URI (base64 string already from Chrome)
    Ok(format!("data:image/png;base64,{base64_png}"))
}

/// Converts HTML to PNG and saves to file
///
/// # Arguments
/// * `html`: HTML content to render
/// * `path`: File path where the PNG will be saved
///
/// # Returns
/// * `Result<()>`: Success or error
pub fn html_to_png_file(html: &str, path: &Path) -> Result<()> {
    html_to_png_file_with(html, path, 16, ConsoleErrorHandling::FailOnErrors)
}

/// Converts HTML to PNG and saves to file with configurable padding and error handling
///
/// This function uses a persistent browser instance for optimal performance and
/// crops the screenshot to the content bounds with the specified padding.
///
/// # Arguments
/// * `html`: HTML content to render
/// * `path`: File path where the PNG will be saved
/// * `padding`: Padding in pixels around the content (0 for tight cropping)
/// * `console_error_handling`: How to handle JavaScript console errors
///
/// # Returns
/// * `Result<()>`: Success or error
pub fn html_to_png_file_with(
    html: &str,
    path: &Path,
    padding: u32,
    console_error_handling: ConsoleErrorHandling,
) -> Result<()> {
    let base64_png = capture_png(&wrap_html(html), padding, console_error_handling)?;

    // Decode base64 to bytes for file writing
    let png_bytes = BASE64
        .decode(&base64_png)
        .map_err(|error| eyre!("Failed to decode base64 PNG data: {error}"))?;

    // Write to file
    std::fs::write(path, &png_bytes)
        .map_err(|error| eyre!("Failed to write PNG file to {}: {error}", path.display()))?;

    Ok(())
}

/// Converts HTML to PDF and saves to file
///
/// # Arguments
/// * `html`: HTML content to render
/// * `path`: File path where the PDF will be saved
///
/// # Returns
/// * `Result<()>`: Success or error
pub fn html_to_pdf(html: &str, path: &Path) -> Result<()> {
    html_to_pdf_with(html, path, ConsoleErrorHandling::FailOnErrors)
}

/// Converts HTML to PDF and saves to file with configurable error handling
///
/// # Arguments
/// * `html`: HTML content to render
/// * `path`: File path where the PDF will be saved
/// * `console_error_handling`: How to handle JavaScript console errors
///
/// # Returns
/// * `Result<()>`: Success or error
pub fn html_to_pdf_with(
    html: &str,
    path: &Path,
    console_error_handling: ConsoleErrorHandling,
) -> Result<()> {
    let pdf_bytes = capture_pdf(html, console_error_handling)?;

    // Write to file
    std::fs::write(path, &pdf_bytes)
        .map_err(|error| eyre!("Failed to write PDF file to {}: {error}", path.display()))?;

    Ok(())
}

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
static BROWSER_MANAGER: LazyLock<Mutex<BrowserManager>> =
    LazyLock::new(|| Mutex::new(BrowserManager::new()));

/// Console error collector for capturing JavaScript errors and console messages
#[derive(Debug, Clone)]
pub struct ConsoleError {
    pub level: String,
    pub message: String,
    pub timestamp: Option<f64>,
}

/// Shared console error collector
type ConsoleErrorCollector = Arc<Mutex<Vec<ConsoleError>>>;

/// Configuration for handling JavaScript console errors
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConsoleErrorHandling {
    /// Log console errors as warnings but don't fail the operation (default)
    #[default]
    LogWarnings,

    /// Fail the operation if any console errors are detected
    FailOnErrors,

    /// Ignore console errors completely
    Ignore,
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
        // Set headless based on BROWSER_OPEN_SECS constant
        // If BROWSER_OPEN_SECS > 0, run in non-headless mode for debugging
        .headless(if cfg!(debug_assertions) {
            BROWSER_OPEN_SECS == 0
        } else {
            true
        })
        .args(args)
        .build()
        .map_err(|error| eyre!("Failed to build browser launch options: {error}"))?;

    let browser = Browser::new(options)
        .map_err(|error| eyre!("Failed to create browser instance: {error}"))?;

    let version = browser
        .get_version()
        .map_err(|error| eyre!("Unable to get version: {error}"))?;
    tracing::debug!("Browser version: {version:?}");

    // Warn user if Chrome/Chromium version does not support all the features we
    // use it for. See https://developer.chrome.com/release-notes
    // Page margin boxes support was introduced in 131
    const CHROME_MIN_VERSION: u32 = 131;

    // Parse and check Chrome version
    if let Some(version_str) = version.product.strip_prefix("Chrome/")
        && let Some(major_version) = version_str.split('.').next()
        && let Ok(major) = major_version.parse::<u32>()
        && major < CHROME_MIN_VERSION
    {
        tracing::warn!(
            "Chrome version {major} is below minimum required version {CHROME_MIN_VERSION}. Some features may not work correctly. Please update Chrome/Chromium."
        );
    }

    Ok(browser)
}

/// Ensures we have a working browser and tab instance, recreating if necessary
fn ensure_browser_available() -> Result<()> {
    const CHROME: &str = "chrome";
    const CHROMIUM: &str = "chromium";
    if !(is_installed(CHROME)? || is_installed(CHROMIUM)?) {
        if is_installable(CHROMIUM)? {
            install_sync(CHROMIUM)?;
        } else {
            bail!("Please install either Chrome/Chromium for this functionality")
        }
    }

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

/// Check if HTML contains a Stencila node that is a JavaScript based visualization
fn has_js_viz(html: &str) -> bool {
    html.contains("<stencila-image-object")
        && (html.contains("application/vnd.plotly.v1+json")
            || html.contains("application/vnd.apache.echarts+json")
            || html.contains("application/vnd.cytoscape.v3+json")
            || html.contains("application/vnd.vega.v5+json")
            || html.contains("application/vnd.vegalite.v5+json")
            || html.contains("text/vnd.mermaid")
            || (html.contains("text/html") && html.contains("<iframe")))
}

/// Check if HTML contains Stencila nodes that require JavaScript for rendering
fn needs_view(html: &str) -> bool {
    has_js_viz(html) || html.contains("<stencila-code-block")
}

/// Configuration for screenshot wait strategies
///
/// This configuration allows selective enabling of different wait strategies to
/// ensure page content is fully rendered before taking screenshots. Strategies
/// include:
///
/// - **Network Idle**: Waits until no network requests for 500ms (like
///   Puppeteer's networkidle0)
/// - **Animation Frame**: Uses double requestAnimationFrame to ensure
///   animations settle
/// - **Web Components**: Waits for custom elements and shadow DOM to initialize  
/// - **DOM Mutations**: Monitors for DOM stability using MutationObserver
/// - **Images and Fonts**: Waits for all images to load and web fonts to be
///   ready
/// - **Iframe Detection**: Ensures all embedded iframes finish loading their
///   content
#[derive(Debug, Clone)]
pub struct WaitConfig {
    pub network_idle: bool,
    pub animation_frame: bool,
    pub web_components: bool,
    pub dom_mutations: bool,
    pub images_and_fonts: bool,
    pub iframe_detection: bool,
    pub timeout: Duration,
    pub mutation_quiet_period: Duration,
}

impl Default for WaitConfig {
    fn default() -> Self {
        Self {
            network_idle: true,
            animation_frame: true,
            web_components: true,
            dom_mutations: true,
            images_and_fonts: true,
            iframe_detection: true,
            timeout: Duration::from_secs(30),
            mutation_quiet_period: Duration::from_millis(500),
        }
    }
}

/// Screenshot waiter that implements multiple wait strategies for ensuring
/// page content is fully rendered before capturing screenshots.
///
/// This replaces the previous hardcoded media-type specific timeouts with
/// more robust, general-purpose waiting strategies that can handle:
/// - Complex interactive visualizations (Plotly, Mermaid, Vega-Lite)
/// - Leaflet maps and other iframe content  
/// - Web fonts and custom typography
/// - CSS animations and transitions
/// - Dynamically loaded content
pub struct ScreenshotWaiter {
    config: WaitConfig,
}

impl ScreenshotWaiter {
    pub fn new(config: WaitConfig) -> Self {
        Self { config }
    }

    /// Wait for page to be ready according to configured strategies
    pub fn wait_for_ready(&self, tab: &Arc<Tab>) -> Result<()> {
        let start_time = Instant::now();

        // Apply each strategy based on configuration
        if self.config.network_idle {
            self.wait_network_idle(tab)?;
        }

        if self.config.images_and_fonts {
            self.wait_images_and_fonts(tab)?;
        }

        if self.config.iframe_detection {
            self.wait_iframes_loaded(tab)?;
        }

        if self.config.web_components {
            self.wait_web_components(tab)?;
        }

        if self.config.dom_mutations {
            self.wait_dom_stability(tab)?;
        }

        if self.config.animation_frame {
            self.wait_animation_frame(tab)?;
        }

        // Check timeout
        if start_time.elapsed() > self.config.timeout {
            return Err(eyre!("Screenshot wait timeout exceeded"));
        }

        Ok(())
    }

    /// Wait for network to be idle (no pending requests)
    ///
    /// Uses Chrome's Page lifecycle events to detect when network activity
    /// has stopped for at least 500ms, similar to Puppeteer's networkidle0.
    fn wait_network_idle(&self, tab: &Arc<Tab>) -> Result<()> {
        // Basic network idle detection - wait for navigation to complete
        // This covers the common case of waiting for main document and resources
        if let Err(error) = tab.wait_until_navigated() {
            tracing::warn!("Network idle wait failed: {error}. Continuing anyway.");
        }

        // Additional wait for any remaining async content with timeout
        let network_idle_check = r#"
            Promise.race([
                new Promise(resolve => {
                    if (document.readyState === 'complete') {
                        // Wait briefly for any async operations
                        setTimeout(resolve, 300);
                    } else {
                        window.addEventListener('load', () => {
                            setTimeout(resolve, 300);
                        });
                        // Fallback timeout if load event never fires
                        setTimeout(resolve, 2000);
                    }
                }),
                // Global timeout
                new Promise(resolve => setTimeout(resolve, 3000))
            ])
        "#;

        if let Err(error) = tab.evaluate(network_idle_check, true) {
            tracing::warn!("Network idle evaluation failed: {error}");
        }

        Ok(())
    }

    /// Wait for all images and web fonts to load
    ///
    /// Uses the CSS Font Loading API (document.fonts.ready) and monitors
    /// image load events to ensure visual content is fully available.
    fn wait_images_and_fonts(&self, tab: &Arc<Tab>) -> Result<()> {
        let images_fonts_script = r#"
            Promise.race([
                (async () => {
                    // Wait for fonts using CSS Font Loading API with timeout
                    try {
                        await Promise.race([
                            document.fonts.ready,
                            new Promise(resolve => setTimeout(resolve, 2000))
                        ]);
                    } catch (e) {
                        console.warn('Font loading check failed:', e);
                    }
                    
                    // Wait for all images to load with shorter timeout
                    const images = Array.from(document.querySelectorAll('img'));
                    if (images.length > 0) {
                        await Promise.all(images.map(img => {
                            if (img.complete) return Promise.resolve();
                            return new Promise((resolve) => {
                                img.addEventListener('load', resolve);
                                img.addEventListener('error', resolve); // Resolve even on error
                                // Shorter timeout per image
                                setTimeout(resolve, 2000);
                            });
                        }));
                    }
                    
                    // Quick check for background images in stylesheets
                    try {
                        const stylesheets = Array.from(document.styleSheets);
                        for (const sheet of stylesheets) {
                            try {
                                // Just check if we can access rules
                                const rules = sheet.cssRules;
                            } catch (e) {
                                // Cross-origin stylesheets will throw, ignore
                            }
                        }
                    } catch (e) {
                        // Ignore stylesheet errors
                    }
                })(),
                // Global timeout for the entire operation
                new Promise(resolve => setTimeout(resolve, 4000))
            ])
        "#;

        if let Err(error) = tab.evaluate(images_fonts_script, true) {
            tracing::warn!("Images and fonts wait failed: {error}");
        }

        Ok(())
    }

    /// Wait for all iframes to finish loading their content
    ///
    /// Handles both same-origin and cross-origin iframes by monitoring
    /// load events. Critical for content like Leaflet maps in iframes.
    fn wait_iframes_loaded(&self, tab: &Arc<Tab>) -> Result<()> {
        let iframe_script = r#"
            Promise.race([
                new Promise(resolve => {
                    const iframes = Array.from(document.querySelectorAll('iframe'));
                    if (iframes.length === 0) {
                        resolve();
                        return;
                    }
                    
                    let pendingFrames = iframes.length;
                    const checkFrame = (iframe) => {
                        const decrementAndCheck = () => {
                            pendingFrames--;
                            if (pendingFrames === 0) resolve();
                        };
                        
                        try {
                            // For same-origin iframes, check readyState
                            if (iframe.contentDocument && iframe.contentDocument.readyState === 'complete') {
                                decrementAndCheck();
                                return;
                            }
                        } catch (e) {
                            // Cross-origin access denied, rely on load event
                        }
                        
                        // Listen for load event
                        iframe.addEventListener('load', decrementAndCheck, { once: true });
                        // Shorter timeout for problematic iframes
                        setTimeout(decrementAndCheck, 4000);
                    };
                    
                    iframes.forEach(checkFrame);
                    
                    // Watch for dynamically added iframes for a shorter period
                    const observer = new MutationObserver(mutations => {
                        mutations.forEach(mutation => {
                            mutation.addedNodes.forEach(node => {
                                if (node.tagName === 'IFRAME') {
                                    pendingFrames++;
                                    checkFrame(node);
                                }
                            });
                        });
                    });
                    
                    observer.observe(document.body, { childList: true, subtree: true });
                    
                    // Stop observing after shorter time
                    setTimeout(() => {
                        observer.disconnect();
                        if (pendingFrames > 0) resolve(); // Force resolve
                    }, 1500);
                }),
                // Global timeout
                new Promise(resolve => setTimeout(resolve, 5000))
            ])
        "#;

        if let Err(error) = tab.evaluate(iframe_script, true) {
            tracing::warn!("Iframe loading wait failed: {error}");
        }

        Ok(())
    }

    /// Wait for web components to be ready
    ///
    /// Specifically handles Stencila components and other custom elements
    /// by checking for common readiness patterns and shadow DOM initialization.
    fn wait_web_components(&self, tab: &Arc<Tab>) -> Result<()> {
        let components_script = r#"
            Promise.race([
                (async () => {
                    // Wait for stencila-image-object to be defined with timeout
                    if (!customElements.get('stencila-image-object')) {
                        try {
                            await Promise.race([
                                customElements.whenDefined('stencila-image-object'),
                                new Promise((_, reject) => setTimeout(() => reject(new Error('timeout')), 2000))
                            ]);
                        } catch (e) {
                            // Component not defined within timeout, continue
                            return;
                        }
                    }
                    
                    // Quick check for component readiness
                    const imageObjects = document.querySelectorAll('stencila-image-object');
                    if (imageObjects.length === 0) return;
                    
                    // Wait briefly for shadow DOM to initialize
                    await new Promise(resolve => setTimeout(resolve, 100));
                    
                    // Check if components have shadow DOM content
                    let allReady = true;
                    imageObjects.forEach(el => {
                        if (!el.shadowRoot || el.shadowRoot.children.length === 0) {
                            allReady = false;
                        }
                    });
                    
                    if (!allReady) {
                        // Wait a bit more for components to render
                        await new Promise(resolve => setTimeout(resolve, 500));
                    }
                })(),
                // Global timeout for the entire operation
                new Promise(resolve => setTimeout(resolve, 3000))
            ])
        "#;

        if let Err(error) = tab.evaluate(components_script, true) {
            tracing::warn!("Web components wait failed: {error}");
        }

        Ok(())
    }

    /// Wait for DOM to stop mutating
    ///
    /// Uses MutationObserver to detect when DOM modifications have stopped
    /// for the configured quiet period, indicating dynamic content has settled.
    fn wait_dom_stability(&self, tab: &Arc<Tab>) -> Result<()> {
        let quiet_period_ms = self.config.mutation_quiet_period.as_millis() as u64;

        let mutation_script = format!(
            r#"
            new Promise(resolve => {{
                let timeout;
                const observer = new MutationObserver(() => {{
                    clearTimeout(timeout);
                    timeout = setTimeout(() => {{
                        observer.disconnect();
                        resolve();
                    }}, {quiet_period_ms});
                }});
                
                observer.observe(document.body, {{
                    childList: true,
                    subtree: true,
                    attributes: true,
                    characterData: true
                }});
                
                // Initial timeout in case no mutations occur
                timeout = setTimeout(() => {{
                    observer.disconnect();
                    resolve();
                }}, {quiet_period_ms});
            }})
            "#
        );

        if let Err(error) = tab.evaluate(&mutation_script, true) {
            tracing::warn!("DOM stability wait failed: {error}");
        }

        Ok(())
    }

    /// Wait for animation frames to stabilize
    ///
    /// Uses double requestAnimationFrame technique to ensure the browser's
    /// rendering pipeline has processed all pending animations and reflows.
    fn wait_animation_frame(&self, tab: &Arc<Tab>) -> Result<()> {
        let animation_script = r#"
            new Promise(resolve => {
                // Double requestAnimationFrame ensures rendering pipeline is flushed
                requestAnimationFrame(() => {
                    requestAnimationFrame(() => {
                        // Additional setTimeout to ensure we're after paint
                        setTimeout(resolve, 0);
                    });
                });
            })
        "#;

        if let Err(error) = tab.evaluate(animation_script, true) {
            tracing::warn!("Animation frame wait failed: {error}");
        }

        Ok(())
    }
}

impl WaitConfig {
    /// Create a configuration optimized for the given HTML content
    ///
    /// Analyzes the HTML to determine which wait strategies are most relevant,
    /// providing intelligent defaults while allowing for customization.
    pub fn for_content(html: &str) -> Self {
        // Check for specific content types that need special handling
        let has_plotly = html.contains("application/vnd.plotly.v1+json");
        let has_mermaid = html.contains("text/vnd.mermaid");
        let has_vega = html.contains("vegalite") || html.contains("application/vnd.vega");
        let has_cytoscape = html.contains("application/vnd.cytoscape");
        let has_stencila_components = html.contains("<stencila-");
        let has_datatable = html.contains("<stencila-datatable");
        let has_iframes = html.contains("<iframe");
        let has_images = html.contains("<img");
        let needs_view = needs_view(html);

        // Determine timeout based on content complexity
        let timeout = if has_iframes {
            Duration::from_secs(15)
        } else if has_plotly {
            Duration::from_secs(10)
        } else if has_mermaid {
            Duration::from_secs(8)
        } else if has_vega || has_cytoscape {
            Duration::from_secs(6)
        } else if needs_view {
            Duration::from_secs(4)
        } else if has_datatable || has_stencila_components {
            Duration::from_secs(3)
        } else {
            Duration::from_secs(2)
        };

        Self {
            // Enable network idle for dynamic content, iframes, or components that might load resources
            network_idle: needs_view || has_iframes || has_stencila_components,

            // Always enable animation frame - it's very fast and ensures stability
            animation_frame: true,

            // Enable web components for any Stencila content, not just dynamic media types
            web_components: has_stencila_components,

            // Enable DOM mutations for dynamic content or components that modify DOM
            dom_mutations: needs_view || has_datatable,

            // Enable images/fonts if we have images, dynamic content, or components that might load fonts
            images_and_fonts: has_images || needs_view || has_stencila_components,

            // Only enable iframe detection if iframes are present
            iframe_detection: has_iframes,

            timeout,

            // Shorter mutation quiet period for faster response
            mutation_quiet_period: Duration::from_millis(300),
        }
    }
}

/// Wait for rendering completion using modern wait strategies
///
/// This function replaces the previous hardcoded media-type timeouts with
/// a more robust approach that uses multiple wait strategies to ensure
/// content is fully rendered. The strategies are intelligently configured
/// based on the HTML content.
fn detect_rendering_completion(tab: &Arc<Tab>, html: &str) -> Result<()> {
    // Create optimized wait configuration based on HTML content
    let config = WaitConfig::for_content(html);

    tracing::debug!(
        "Using wait strategies: network_idle={}, animation_frame={}, web_components={}, dom_mutations={}, images_and_fonts={}, iframe_detection={}",
        config.network_idle,
        config.animation_frame,
        config.web_components,
        config.dom_mutations,
        config.images_and_fonts,
        config.iframe_detection
    );

    // Apply the wait strategies
    let waiter = ScreenshotWaiter::new(config);
    if let Err(error) = waiter.wait_for_ready(tab) {
        tracing::debug!("Screenshot wait strategies failed: {error}. Taking screenshot anyway.");
    } else {
        tracing::trace!("All wait strategies completed successfully");
    }

    Ok(())
}

/// Wait for content bounds to be detected with exponential backoff
///
/// If content bounds cannot be detected immediately, this indicates that
/// rendering is not yet complete. We use exponential backoff to wait longer
/// between attempts, up to a maximum timeout of 30 seconds.
fn wait_for_content_bounds_with_backoff(
    tab: &Arc<Tab>,
    html: &str,
    padding: u32,
) -> Result<Option<Page::Viewport>> {
    let start_time = Instant::now();
    let max_wait = Duration::from_secs(10); // Reduced from 30s since logic is simpler
    let mut attempt = 0;

    loop {
        // Try to get content bounds
        match get_content_bounds(tab, padding) {
            Ok(Some(bounds)) => {
                tracing::debug!(
                    "Content bounds detected on attempt {}: {}×{} at ({}, {})",
                    attempt + 1,
                    bounds.width,
                    bounds.height,
                    bounds.x,
                    bounds.y
                );
                return Ok(Some(bounds));
            }
            Ok(None) => {
                // Content bounds not available yet, check if we should continue waiting
                let elapsed = start_time.elapsed();
                if elapsed >= max_wait {
                    tracing::debug!(
                        "Content bounds not detected after {}s, using full page screenshot",
                        elapsed.as_secs()
                    );
                    return Ok(None);
                }

                // Calculate exponential backoff delay (50ms, 100ms, 200ms, 400ms, 800ms, 1600ms, max 3200ms)
                let delay_ms = (50 * (1 << attempt)).min(3200);
                tracing::debug!(
                    "Content bounds not detected on attempt {}, waiting {}ms before retry",
                    attempt + 1,
                    delay_ms
                );

                sleep(Duration::from_millis(delay_ms));

                // Additional wait strategy: if this is a Stencila component, give it more time on first attempt
                if attempt == 0 && html.contains("stencila-") {
                    tracing::debug!(
                        "Stencila component detected, applying additional wait for rendering"
                    );
                    sleep(Duration::from_millis(500)); // Simple additional wait
                }

                attempt += 1;
            }
            Err(error) => {
                tracing::warn!(
                    "Failed to get content bounds: {error}. Using full page screenshot."
                );
                return Ok(None);
            }
        }
    }
}

/// Calculate the bounding box of the content wrapper for cropping
fn get_content_bounds(tab: &Arc<Tab>, padding: u32) -> Result<Option<Page::Viewport>> {
    let bounds_js = format!(
        r#"
        new Promise((resolve) => {{
            const wrapper = document.getElementById('stencila-content-wrapper');
            if (!wrapper) {{
                resolve(null);
                return;
            }}
            
            // Wait for wrapper to have content
            if (wrapper.children.length === 0) {{
                resolve(null);
                return;
            }}
            
            const rect = wrapper.getBoundingClientRect();
            const padding = {padding};
            
            // Wait for reasonable dimensions (at least 10x10 pixels to account for small content)
            if (rect.width < 10 || rect.height < 10) {{
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
    // Use local or production web assets based on USE_LOCALHOST constant
    let web_base = if USE_LOCALHOST {
        "http://localhost:9000/~static/dev".to_string()
    } else {
        ["https://stencila.io/web/v", STENCILA_VERSION].concat()
    };

    // Add theme CSS
    // Base theme is always needed for application of variables. Get from web dist, falling back to remote.
    let base = if let Some(file) = Web::get("themes/base.css") {
        let content = String::from_utf8_lossy(&file.data);
        format!(r#"<style>{content}</style>"#)
    } else {
        format!(r#"<link rel="stylesheet" type="text/css" href="{web_base}/themes/base.css">"#)
    };
    // Custom theme, falling back to Stencila
    // Note that the resolved theme is based on the output path
    // TODO: support getting theme by name and path, will currently use cwd
    let overrides = if let Ok(Some(theme)) = stencila_themes::get_sync(None, None) {
        // This uses the theme's computed CSS variables, rather than injecting the content of the theme
        // because we found that `color-mix` was not supported in headless chrome and so was breaking
        // the calculation of theme variables and rendering of Mermaid and other JS-based images.
        let content = theme.computed_css(LengthConversion::KeepUnits);
        format!(r#"<style>{content}</style>"#)
    } else if let Some(file) = Web::get("themes/stencila.css") {
        let content = String::from_utf8_lossy(&file.data);
        format!(r#"<style>{content}</style>"#)
    } else {
        format!(r#"<link rel="stylesheet" type="text/css" href="{web_base}/themes/stencila.css">"#)
    };
    let theme = [base, overrides].concat();

    // Add CSS for the wrapper element
    let wrapper_css = if has_js_viz(html) {
        // For JS-based visualizations we need to set a width because they
        // expand to that. We use the current default value for `--plot-width` in plots.css
        "display: block;
        width: 8in;"
    } else {
        "display: block;
        width: fit-content;
        height: fit-content;"
    };

    // Add static view JavaScript and CSS if necessary - only when HTML contains
    // interactive visualizations that require dynamic module loading (Plotly,
    // Mermaid, Vega-Lite, etc.) Due to the way that modules are dynamically,
    // asynchronously loaded it is not possible to inject these, they must come
    // from a server.
    let scripts = if needs_view(html) {
        format!(
            r#"
            <link rel="stylesheet" type="text/css" href="{web_base}/views/static.css">
            <script type="module" src="{web_base}/views/static.js"></script>
        "#
        )
    } else {
        String::new()
    };

    // Remove attributes added to top level HTML nodes to prevent node cards being
    // shown by static view. Ideally web component cards would only open by default
    // for [root] nodes. But for now this is best approach.
    let html = html.replace(" depth=0 ancestors=''", "");

    format!(
        r#"<!doctype html>
<html lang="en">
    <head>
        <meta charset="utf-8"/>
        <title>Stencila Screen</title>
        <meta name="viewport" content="width=device-width, initial-scale=1" />
        {theme}
        <style>
            body {{
                margin: 0;
                padding: 0;
                font-family: system-ui, -apple-system, sans-serif;
            }}
            #stencila-content-wrapper {{
                {wrapper_css}
            }}
        </style>
        {scripts}
    </head>
    <body>
        <div id="stencila-content-wrapper" view="static">
            {html}
        </div>
    </body>
</html>"#
    )
}

/// Captures HTML content as PNG using persistent browser instance with console error handling
fn capture_png(
    html: &str,
    padding: u32,
    console_error_handling: ConsoleErrorHandling,
) -> Result<String> {
    // Testing indicated that for Plotly is is best to use a fresh browser tab
    if html.contains("application/vnd.plotly.v1+json") {
        shutdown().ok();
    }

    // Ensure we have a working browser, recreating if necessary
    ensure_browser_available()?;

    let result = try_png(html, padding, console_error_handling);
    if result.is_err() {
        // Force recreation by clearing both browser and tab
        if let Ok(mut manager) = BROWSER_MANAGER.lock() {
            manager.clear_browser_and_tab();
        }

        // Ensure browser and tab are available again
        ensure_browser_available()?;

        // Retry the screenshot
        try_png(html, padding, console_error_handling)
    } else {
        result
    }
}

/// Attempts to take a screenshot with console error handling
fn try_png(
    html: &str,
    padding: u32,
    console_error_handling: ConsoleErrorHandling,
) -> Result<String> {
    // Set up the browser tab with the HTML content
    let (mut manager, tab) = setup_tab_for_capture(html)?;

    // Set up console error handling
    let console_errors = setup_console_error_handling(&tab, console_error_handling)?;

    // Wait for interactive visualizations to complete rendering
    if let Err(error) = detect_rendering_completion(&tab, html) {
        tracing::warn!("Rendering completion detection failed: {error}. Taking screenshot anyway.");
    }

    // Calculate content bounds for cropping with exponential backoff
    let clip = wait_for_content_bounds_with_backoff(&tab, html, padding)?;

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

    // Handle console errors based on configuration
    handle_console_errors(console_errors, console_error_handling, "rendering")?;

    // Keep browser open for inspection if BROWSER_OPEN_SECS > 0
    #[cfg(debug_assertions)]
    #[allow(clippy::absurd_extreme_comparisons)]
    if BROWSER_OPEN_SECS > 0 {
        tracing::info!("Browser staying open for {BROWSER_OPEN_SECS} seconds for inspection...");
        sleep(Duration::from_secs(BROWSER_OPEN_SECS));
    }

    // Cleanup
    manager.cleanup();

    Ok(result.data)
}

/// Converts HTML to PDF using the current browser instance with console error handling
fn capture_pdf(html: &str, console_error_handling: ConsoleErrorHandling) -> Result<Vec<u8>> {
    // Ensure we have a working browser, recreating if necessary
    ensure_browser_available()?;

    let result = try_pdf(html, console_error_handling);
    if result.is_err() {
        // Force recreation by clearing both browser and tab
        if let Ok(mut manager) = BROWSER_MANAGER.lock() {
            manager.clear_browser_and_tab();
        }

        // Ensure browser and tab are available again
        ensure_browser_available()?;

        // Retry the PDF capture
        try_pdf(html, console_error_handling)
    } else {
        result
    }
}

/// Attempts to generate a PDF with console error handling
fn try_pdf(html: &str, console_error_handling: ConsoleErrorHandling) -> Result<Vec<u8>> {
    // Set up the browser tab with the HTML content
    let (mut manager, tab) = setup_tab_for_capture(html)?;

    // Set up console error handling
    let console_errors = setup_console_error_handling(&tab, console_error_handling)?;

    // Wait for interactive visualizations to complete rendering
    if let Err(error) = detect_rendering_completion(&tab, html) {
        tracing::warn!("Rendering completion detection failed: {error}. Generating PDF anyway.");
    }

    // Generate PDF
    let pdf_bytes = tab
        .print_to_pdf(Some(PrintToPdfOptions {
            // Make sure that @page rules are observed (this option is not just about page size)
            prefer_css_page_size: Some(true),
            ..Default::default()
        }))
        .map_err(|error| eyre!("Failed to generate PDF: {error}"))?;

    tracing::debug!("PDF generated successfully");

    // Handle console errors based on configuration
    handle_console_errors(console_errors, console_error_handling, "PDF generation")?;

    // Keep browser open for inspection if BROWSER_OPEN_SECS > 0
    #[cfg(debug_assertions)]
    #[allow(clippy::absurd_extreme_comparisons)]
    if BROWSER_OPEN_SECS > 0 {
        tracing::info!("Browser staying open for {BROWSER_OPEN_SECS} seconds for inspection...");
        sleep(Duration::from_secs(BROWSER_OPEN_SECS));
    }

    // Cleanup
    manager.cleanup();

    Ok(pdf_bytes)
}

/// Sets up a browser tab with the given HTML content and static mode
/// Returns the browser manager and tab for further operations
fn setup_tab_for_capture(
    html: &str,
) -> Result<(std::sync::MutexGuard<'static, BrowserManager>, Arc<Tab>)> {
    let manager = BROWSER_MANAGER
        .lock()
        .map_err(|error| eyre!("Failed to acquire browser manager lock: {error}"))?;

    let tab = manager
        .tab()
        .ok_or_else(|| eyre!("No tab instance available"))?
        .clone();

    // Ensure light color schema
    let html = html.replace(
        r#"<html lang="en">"#,
        r#"<html lang="en" data-color-scheme="light">"#,
    );

    // Use Page.setDocumentContent for fastest content injection
    let frame_tree = tab
        .call_method(Page::GetFrameTree(None))
        .map_err(|error| eyre!("Failed to get frame tree: {error}"))?;
    tab.call_method(Page::SetDocumentContent {
        frame_id: frame_tree.frame_tree.frame.id,
        html,
    })
    .map_err(|error| eyre!("Failed to set document content: {error}"))?;

    Ok((manager, tab))
}

/// Sets up console error handling for a browser tab
/// Returns an optional console error collector based on the handling mode
fn setup_console_error_handling(
    tab: &Arc<Tab>,
    console_error_handling: ConsoleErrorHandling,
) -> Result<Option<ConsoleErrorCollector>> {
    if console_error_handling == ConsoleErrorHandling::Ignore {
        return Ok(None);
    }

    tab.enable_log()
        .map_err(|error| eyre!("Failed to enable log: {error}"))?;

    // Enable Runtime domain to receive console API calls and exceptions
    tab.call_method(Runtime::Enable(Default::default()))
        .map_err(|error| eyre!("Failed to enable runtime: {error}"))?;

    // Create console error collector for this session
    let console_errors: ConsoleErrorCollector = Arc::new(Mutex::new(Vec::new()));
    let console_errors_clone = Arc::clone(&console_errors);

    tab.add_event_listener(Arc::new(move |event: &Event| {
        match event {
            Event::LogEntryAdded(entry) => {
                tracing::debug!("{:?} {}", entry.params.entry.level, entry.params.entry.text)
            }
            Event::RuntimeConsoleAPICalled(console_event) => {
                let level = format!("{:?}", console_event.params.Type).to_lowercase();
                let mut message = String::new();

                // Combine all arguments into a single message
                for arg in &console_event.params.args {
                    if let Some(value) = &arg.value {
                        message.push_str(&format!("{} ", value));
                    }
                }

                // Only capture error and assert level messages
                if (level.contains("error") || level.contains("assert"))
                    && let Ok(mut errors) = console_errors_clone.lock()
                {
                    errors.push(ConsoleError {
                        level: level.clone(),
                        message: message.trim().to_string(),
                        timestamp: Some(console_event.params.timestamp),
                    });
                }

                tracing::debug!("Console {}: {}", level, message);
            }
            Event::RuntimeExceptionThrown(exception_event) => {
                let exception_details = &exception_event.params.exception_details;

                // Build a comprehensive error message with all available details
                let mut message_parts = Vec::new();

                // Basic exception text
                message_parts.push(exception_details.text.clone());

                // Add line and column information if available
                if exception_details.line_number > 0 {
                    if exception_details.column_number > 0 {
                        message_parts.push(format!(
                            "at line {}, column {}",
                            exception_details.line_number, exception_details.column_number
                        ));
                    } else {
                        message_parts.push(format!("at line {}", exception_details.line_number));
                    }
                }

                // Add script/URL information if available
                if let Some(url) = &exception_details.url {
                    message_parts.push(format!("in {}", url));
                } else if let Some(script_id) = &exception_details.script_id {
                    message_parts.push(format!("in script {}", script_id));
                }

                // Add stack trace if available
                if let Some(stack_trace) = &exception_details.stack_trace
                    && !stack_trace.call_frames.is_empty()
                {
                    let mut stack_info = Vec::new();
                    for (i, frame) in stack_trace.call_frames.iter().take(3).enumerate() {
                        let function_name = if frame.function_name.is_empty() {
                            "<anonymous>"
                        } else {
                            &frame.function_name
                        };
                        let url = if frame.url.is_empty() {
                            "<unknown>"
                        } else {
                            &frame.url
                        };

                        let frame_info = format!(
                            "  {}. {} ({}:{}:{})",
                            i + 1,
                            function_name,
                            url,
                            frame.line_number,
                            frame.column_number
                        );
                        stack_info.push(frame_info);
                    }
                    if stack_trace.call_frames.len() > 3 {
                        stack_info.push("  ... (truncated)".to_string());
                    }
                    message_parts.push(format!("Stack trace:\n{}", stack_info.join("\n")));
                }

                // Add exception object details if available
                if let Some(exception) = &exception_details.exception
                    && let Some(description) = &exception.description
                    && !description.is_empty()
                    && description != &exception_details.text
                {
                    message_parts.push(format!("Exception details: {}", description));
                }

                let comprehensive_message = message_parts.join(" | ");

                if let Ok(mut errors) = console_errors_clone.lock() {
                    errors.push(ConsoleError {
                        level: "exception".to_string(),
                        message: comprehensive_message.clone(),
                        timestamp: None,
                    });
                }

                tracing::debug!("JavaScript exception: {}", comprehensive_message);
            }
            _ => {}
        }
    }))
    .map_err(|error| eyre!("Failed to add event listener: {error}"))?;

    Ok(Some(console_errors))
}

/// Handles console errors based on configuration
/// Returns an error if FailOnErrors mode is set and errors were found
fn handle_console_errors(
    console_errors: Option<ConsoleErrorCollector>,
    console_error_handling: ConsoleErrorHandling,
    context: &str,
) -> Result<()> {
    if let Some(console_errors) = console_errors
        && let Ok(errors) = console_errors.lock()
        && !errors.is_empty()
    {
        let error_messages: Vec<String> = errors
            .iter()
            .map(|e| format!("[{}] {}", e.level, e.message))
            .collect();

        match console_error_handling {
            ConsoleErrorHandling::LogWarnings => {
                tracing::warn!(
                    "JavaScript console errors detected during {}: {}",
                    context,
                    error_messages.join("; ")
                );
            }
            ConsoleErrorHandling::FailOnErrors => {
                bail!(
                    "JavaScript console errors detected during {}: {}",
                    context,
                    error_messages.join("; ")
                );
            }
            ConsoleErrorHandling::Ignore => {
                // Should never reach here due to early return in setup
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{
        fs::{metadata, remove_file},
        path::PathBuf,
        time::{Duration, Instant},
    };

    use super::*;

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
        assert!(!needs_view(simple_html));

        // HTML with stencila-image-object but no dynamic media types should not need scripts
        let static_html =
            r#"<stencila-image-object media-type="image/png">test</stencila-image-object>"#;
        assert!(!needs_view(static_html));

        // HTML with Mermaid should need scripts
        let mermaid_html =
            r#"<stencila-image-object media-type="text/vnd.mermaid">test</stencila-image-object>"#;
        assert!(needs_view(mermaid_html));

        // HTML with Plotly should need scripts
        let plotly_html = r#"<stencila-image-object media-type="application/vnd.plotly.v1+json">test</stencila-image-object>"#;
        assert!(needs_view(plotly_html));

        // HTML with Vega-Lite should need scripts
        let vega_html = r#"<stencila-image-object media-type="application/vnd.vegalite.v5+json">test</stencila-image-object>"#;
        assert!(needs_view(vega_html));
    }

    #[ignore = "primarily for development"]
    #[allow(clippy::dbg_macro)]
    #[test_log::test]
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
    /// RUST_LOG=trace cargo test -p stencila-convert html_to_png::tests::test_rendering -- --nocapture
    #[ignore = "primarily for development"]
    #[test_log::test(tokio::test(flavor = "multi_thread"))]
    async fn test_rendering() -> Result<()> {
        let datatable_html = r#"<stencila-datatable>
            <table>
                <thead>
                <tr>
                    <th>
                    <stencila-datatable-column>
                        <span>a</span>
                    </stencila-datatable-column>
                    </th>
                    <th>
                    <stencila-datatable-column>
                        <span>b</span>
                    </stencila-datatable-column>
                    </th>
                </tr>
                </thead>
                <tbody>
                <tr>
                    <td data-type="number">1</td>
                    <td data-type="number">3</td>
                </tr>
                <tr>
                    <td data-type="number">2</td>
                    <td data-type="number">4</td>
                </tr>
                </tbody>
            </table>
            </stencila-datatable>
            "#;

        let mermaid_html = r#"
            <stencila-image-object 
                media-type="text/vnd.mermaid" 
                content-url="graph TD
    A --> B
">
            </stencila-image-object>
        "#;

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

        let examples = [
            ("test-datatable.png", datatable_html),
            ("test-mermaid.png", mermaid_html),
            ("test-plotly.png", plotly_html),
            ("test-vega-lite.png", vega_html),
        ];

        for (filename, html) in examples {
            let output_path = Path::new(filename);

            if output_path.exists() {
                remove_file(output_path)?;
            }

            let wrapped_html = wrap_html(html);
            html_to_png_file(&wrapped_html, output_path)?;

            let file_size = metadata(output_path)?.len();
            assert!(
                file_size > 1000,
                "PNG file should have substantial content (got {file_size} bytes)"
            );
        }

        Ok(())
    }

    #[ignore = "primarily for development"]
    #[test]
    fn test_html_to_pdf() -> Result<()> {
        let simple_html =
            "<html><body><h1>Test PDF</h1><p>This is a simple test document.</p></body></html>";
        let test_path = PathBuf::from("test_output.pdf");

        // Remove existing file if it exists
        if test_path.exists() {
            remove_file(&test_path).ok();
        }

        // Generate PDF
        html_to_pdf(simple_html, &test_path)?;

        // Verify file was created and has content
        assert!(test_path.exists(), "PDF file should be created");
        let file_size = metadata(&test_path)?.len();
        assert!(file_size > 0, "PDF file should have content");

        // Clean up
        remove_file(&test_path).ok();

        Ok(())
    }

    /// To run this test with logs printed:
    /// RUST_LOG=info cargo test -p stencila-convert html_to_png::tests::test_console_errors -- --nocapture
    #[ignore = "primarily for development"]
    #[test_log::test]
    fn test_console_errors() -> eyre::Result<()> {
        tracing::info!("Testing console error capture...");

        // HTML with intentional JavaScript errors
        let html_with_errors = r#"
    <html>
    <head>
        <script>
            console.log("This should work fine");
            console.error("This is an intentional error");
            console.warn("This is a warning");

            // This will cause an exception
            try {
                nonExistentFunction();
            } catch (e) {
                console.error("Caught exception:", e.message);
            }

            // This will cause an uncaught exception
            setTimeout(() => {
                throw new Error("Uncaught error for testing");
            }, 100);
        </script>
    </head>
    <body>
        <h1>Test Console Errors</h1>
        <p>This page contains intentional JavaScript errors for testing.</p>
    </body>
    </html>
    "#;

        tracing::info!("\n1. Testing with LogWarnings (should succeed but log warnings):");
        match html_to_png_data_uri_with(html_with_errors, 16, ConsoleErrorHandling::LogWarnings) {
            Ok(_) => tracing::info!("LogWarnings mode: Operation succeeded (as expected)"),
            Err(e) => bail!("LogWarnings mode: Unexpected error: {e}"),
        }

        tracing::info!("\n2. Testing with FailOnErrors (should fail due to console errors):");
        match html_to_png_data_uri_with(html_with_errors, 16, ConsoleErrorHandling::FailOnErrors) {
            Ok(_) => bail!("FailOnErrors mode: Unexpected success"),
            Err(e) => tracing::info!("FailOnErrors mode: Failed as expected: {e}"),
        }

        tracing::info!("\n3. Testing with Ignore (should succeed and ignore errors):");
        match html_to_png_data_uri_with(html_with_errors, 16, ConsoleErrorHandling::Ignore) {
            Ok(_) => tracing::info!("Ignore mode: Operation succeeded (as expected)"),
            Err(e) => bail!("Ignore mode: Unexpected error: {e}"),
        }

        // Test with clean HTML (no errors)
        let clean_html = r#"
    <html>
    <head>
        <script>
            console.log("This is a normal log message");
        </script>
    </head>
    <body>
        <h1>Clean Test</h1>
        <p>This page has no JavaScript errors.</p>
    </body>
    </html>
    "#;

        tracing::info!("\n4. Testing clean HTML with FailOnErrors (should succeed):");
        match html_to_png_data_uri_with(clean_html, 16, ConsoleErrorHandling::FailOnErrors) {
            Ok(_) => tracing::info!("Clean HTML: Operation succeeded (as expected)"),
            Err(e) => bail!("Clean HTML: Unexpected error: {e}"),
        }

        tracing::info!("\nConsole error capture test completed!");
        Ok(())
    }
}
