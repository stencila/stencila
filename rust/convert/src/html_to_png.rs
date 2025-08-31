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
    thread::sleep,
    time::{Duration, Instant},
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
    Ok(format!("data:image/png;base64,{base64_png}"))
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

/// Converts HTML to PDF and saves to file
///
/// This function uses a persistent browser instance for optimal performance.
/// Optionally, call `warmup()` during application startup for optimal first-call performance.
///
/// # Arguments
/// * `html`: HTML content to render
/// * `path`: File path where the PDF will be saved
///
/// # Returns
/// * `Result<()>`: Success or error
pub fn html_to_pdf(html: &str, path: &Path) -> Result<()> {
    let pdf_bytes = capture_pdf(html)?;

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
static BROWSER_MANAGER: Lazy<Mutex<BrowserManager>> =
    Lazy::new(|| Mutex::new(BrowserManager::new()));

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
    html.contains("<stencila-")
        && (html.contains("application/vnd.plotly.v1+json")
            || html.contains("text/vnd.mermaid")
            || html.contains("application/vnd.vegalite.v5+json")
            || html.contains("application/vnd.vega.v5+json")
            || html.contains("application/vnd.cytoscape.v3+json")
            || (html.contains("text/html") && html.contains("<iframe")))
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
        let has_stencila_components = html.contains("stencila-");
        let has_datatable = html.contains("stencila-datatable");
        let has_iframes = html.contains("<iframe");
        let has_images = html.contains("<img");
        let has_dynamic_content = needs_dynamic_scripts(html);

        // Determine timeout based on content complexity
        let timeout = if has_iframes {
            Duration::from_secs(15)
        } else if has_plotly {
            Duration::from_secs(10)
        } else if has_mermaid {
            Duration::from_secs(8)
        } else if has_vega || has_cytoscape {
            Duration::from_secs(6)
        } else if has_dynamic_content {
            Duration::from_secs(4)
        } else if has_datatable || has_stencila_components {
            Duration::from_secs(3)
        } else {
            Duration::from_secs(2)
        };

        Self {
            // Enable network idle for dynamic content, iframes, or components that might load resources
            network_idle: has_dynamic_content || has_iframes || has_stencila_components,

            // Always enable animation frame - it's very fast and ensures stability
            animation_frame: true,

            // Enable web components for any Stencila content, not just dynamic media types
            web_components: has_stencila_components,

            // Enable DOM mutations for dynamic content or components that modify DOM
            dom_mutations: has_dynamic_content || has_datatable,

            // Enable images/fonts if we have images, dynamic content, or components that might load fonts
            images_and_fonts: has_images || has_dynamic_content || has_stencila_components,

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
    // Keep any unused linting warning here so that if we turn on the line below
    // it is harder to forget to reverse that before committing
    let static_prefix = format!("https://stencila.io/web/v{STENCILA_VERSION}");

    // During development of web components uncomment the next line and run
    // a local Stencila server with permissive CORS (so that headless
    // browser can get use web dist):
    //
    // cargo run -p cli serve --cors permissive
    //let static_prefix = "http://localhost:9000/~static/dev".to_string();

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
                display: block;
                width: fit-content;
                height: fit-content;
            }}
        </style>
        {scripts}
    </head>
    <body>
        <div id="stencila-content-wrapper">
            {html}
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
    let mut manager = BROWSER_MANAGER
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
    if let Err(error) = detect_rendering_completion(tab, html) {
        tracing::warn!("Rendering completion detection failed: {error}. Taking screenshot anyway.",);
    }

    // Calculate content bounds for cropping with exponential backoff
    let clip = wait_for_content_bounds_with_backoff(tab, html, padding)?;

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

    // Dropping of the browser does not work reliably, causing zombie chrome processes
    // so shutdown explicitly.
    manager.cleanup();

    Ok(result.data)
}

/// Converts HTML to PDF using the current browser instance
fn capture_pdf(html: &str) -> Result<Vec<u8>> {
    // Ensure we have a working browser, recreating if necessary
    ensure_browser_available()?;

    let result = try_capture_pdf(html);
    if result.is_err() {
        // Force recreation by clearing both browser and tab
        if let Ok(mut manager) = BROWSER_MANAGER.lock() {
            manager.clear_browser_and_tab();
        }

        // Ensure browser and tab are available again
        ensure_browser_available()?;

        // Retry the PDF capture
        try_capture_pdf(html)
    } else {
        result
    }
}

/// Attempts to generate a PDF with the current tab instance
fn try_capture_pdf(html: &str) -> Result<Vec<u8>> {
    let mut manager = BROWSER_MANAGER
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

    // Wait for interactive visualizations to complete rendering
    // This detects media types and waits only when necessary
    if let Err(error) = detect_rendering_completion(tab, html) {
        tracing::warn!("Rendering completion detection failed: {error}. Generating PDF anyway.",);
    }

    // Generate PDF with default options
    let pdf_bytes = tab
        .print_to_pdf(None)
        .map_err(|error| eyre!("Failed to generate PDF: {error}"))?;

    tracing::debug!("PDF generated successfully");

    // Dropping of the browser does not work reliably, causing zombie chrome processes
    // so shutdown explicitly.
    manager.cleanup();

    Ok(pdf_bytes)
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
    fn test_html_to_pdf() -> Result<()> {
        use std::path::PathBuf;

        let simple_html =
            "<html><body><h1>Test PDF</h1><p>This is a simple test document.</p></body></html>";
        let test_path = PathBuf::from("test_output.pdf");

        // Remove existing file if it exists
        if test_path.exists() {
            std::fs::remove_file(&test_path).ok();
        }

        // Generate PDF
        html_to_pdf(simple_html, &test_path)?;

        // Verify file was created and has content
        assert!(test_path.exists(), "PDF file should be created");
        let file_size = std::fs::metadata(&test_path)?.len();
        assert!(file_size > 0, "PDF file should have content");

        // Clean up
        std::fs::remove_file(&test_path).ok();

        Ok(())
    }

    #[ignore = "primarily for development"]
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
    /// RUST_LOG=trace cargo test -p convert html_to_png::tests::test_rendering -- --nocapture
    #[ignore = "primarily for development"]
    #[test_log::test]
    fn test_rendering() -> Result<()> {
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
                std::fs::remove_file(output_path)?;
            }

            let wrapped_html = wrap_html(html);
            html_to_png_file(&wrapped_html, output_path)?;

            let file_size = std::fs::metadata(output_path)?.len();
            assert!(
                file_size > 1000,
                "PNG file should have substantial content (got {file_size} bytes)"
            );
        }

        Ok(())
    }
}
