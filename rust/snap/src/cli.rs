//! CLI wrapper for snap command

use std::{path::PathBuf, process::exit};

use clap::Parser;
use eyre::Result;

use stencila_cli_utils::{Code, ToStdout, color_print::cstr};
use stencila_format::Format;

use crate::{MeasureMode, ScreenshotResizeMode, ScreenshotResizePolicy, SnapOptions, snap};

use super::{
    browser::{ColorScheme, WaitConfig, WaitUntil},
    devices::{DevicePreset, ViewportConfig},
    measure::MeasurePreset,
};

const ASSERTION_FAILED: i32 = 12;
const BROWSER_FAILURE: i32 = 40;

/// Capture screenshots and measurements of documents served by Stencila
///
/// The `snap` command allows programmatic screenshotting and measurement of
/// pages served by Stencila. It can be used to:
///
/// - Iterate on themes and styled elements and verify changes
/// - Capture screenshots for documentation or CI
/// - Assert computed CSS properties and layout metrics
/// - Measure page elements for automated testing
/// - Extract resolved CSS custom property (theme token) values
/// - Extract the page's color palette
/// - Batch-measure across multiple device viewports
#[derive(Debug, Parser)]
#[command(after_long_help = CLI_AFTER_LONG_HELP)]
pub struct Cli {
    /// Route or path to snap
    ///
    /// If this value exists as a file on disk, it is treated as a document
    /// path and served directly. Otherwise it is treated as a site route
    /// (e.g., "/" or "/docs/guide/"). Defaults to "/" (site root) when omitted.
    route_or_path: Option<String>,

    /// Screenshot output path (.png)
    ///
    /// If specified, a screenshot will be captured and saved to this path.
    #[arg(long)]
    shot: Option<PathBuf>,

    /// CSS selector to capture or measure
    ///
    /// If specified, screenshots will be cropped to this element and
    /// measurements will focus on it. Overrides the --measure preset selectors
    /// and takes precedence over --full for capture mode.
    #[arg(long)]
    selector: Option<String>,

    /// Capture full scrollable page
    ///
    /// By default, captures only the viewport (first screen). Use this flag
    /// to capture the entire scrollable document.
    #[arg(long)]
    full: bool,

    /// Device preset
    ///
    /// Use a predefined viewport configuration: laptop, desktop, mobile,
    /// tablet, tablet-landscape
    #[arg(long, value_enum)]
    device: Option<DevicePreset>,

    /// Measure at multiple device presets in one invocation
    ///
    /// Comma-separated list of device presets. Results are keyed by device
    /// name in the output. When combined with --shot, screenshots are named
    /// {stem}-{device}.png.
    #[arg(long, value_delimiter = ',')]
    devices: Option<Vec<DevicePreset>>,

    /// Viewport width in pixels
    ///
    /// Overrides device preset width if both are specified
    #[arg(long)]
    width: Option<u32>,

    /// Viewport height in pixels
    ///
    /// Overrides device preset height if both are specified
    #[arg(long)]
    height: Option<u32>,

    /// Device pixel ratio
    ///
    /// Overrides device preset DPR if both are specified
    #[arg(long)]
    dpr: Option<f32>,

    /// Screenshot resize mode: never, auto, optimize
    #[arg(long, value_enum, default_value = "auto")]
    resize: ScreenshotResizeModeArg,

    /// Maximum screenshot dimension in pixels after resize
    #[arg(long)]
    max_image_dimension: Option<u32>,

    /// Use light color scheme
    #[arg(long, conflicts_with_all = ["dark", "print"])]
    light: bool,

    /// Use dark color scheme
    #[arg(long, conflicts_with_all = ["light", "print"])]
    dark: bool,

    /// Preview with print media styles (A4 width, for PDF preview)
    ///
    /// Sets viewport to A4 dimensions (794x1123px), emulates print media type,
    /// and applies simulated page margins (75px, based on @page margins from
    /// web/src/themes/base/pages.css). This provides a preview of PDF output
    /// but is not identical - theme-defined @page margin boxes, custom page
    /// sizes, and other advanced @page features will not be rendered.
    /// Conflicts with --light, --dark, and --device options.
    #[arg(long, conflicts_with_all = ["light", "dark", "device"])]
    print: bool,

    /// When to capture: load, domcontentloaded, networkidle
    #[arg(long, default_value = "network-idle")]
    wait_until: WaitUntil,

    /// Wait for CSS selector to exist before capturing
    #[arg(long)]
    wait_for: Option<String>,

    /// Additional delay in milliseconds after page is ready
    #[arg(long)]
    delay: Option<u64>,

    /// Collect computed CSS and layout metrics
    ///
    /// When used without a value, auto-selects the preset based on the target:
    /// "site" for routes, "document" for file paths.
    ///
    /// Presets:
    ///   document - document content selectors (stencila-article, headings, etc.)
    ///   site     - site chrome selectors (layout, header, nav, logo, sidebar, footer)
    ///   all      - both document and site selectors
    ///   header   - header and top-bar selectors
    ///   nav      - navigation and breadcrumb selectors
    ///   main     - main content selectors
    ///   footer   - footer selectors
    ///   theme    - combined theme review selectors across key regions
    #[arg(long, value_enum, num_args = 0..=1, default_missing_value = "auto")]
    measure: Option<MeasurePresetArg>,

    /// Extract resolved CSS custom property (theme token) values
    ///
    /// Reads all --* custom properties from stylesheets and returns their
    /// computed values, grouped by token family. Useful for verifying theme
    /// token resolution and narrowing output with --token-prefix.
    #[arg(long)]
    tokens: bool,

    /// Filter extracted tokens by CSS custom property prefix
    ///
    /// Accepts values with or without the leading `--`. Can be repeated or
    /// provided as a comma-separated list, e.g. `--token-prefix color,font`.
    #[arg(long, value_delimiter = ',', requires = "tokens")]
    token_prefix: Vec<String>,

    /// Extract the page's color palette
    ///
    /// Samples computed color, background-color, and border-color from all
    /// visible elements and returns unique hex values sorted by usage count.
    #[arg(long)]
    palette: bool,

    /// Assert measurement conditions
    ///
    /// Can be specified multiple times. Each assertion is a condition like:
    /// - css(.title).paddingTop >= 24px
    /// - count(section) == 5
    /// - box(.header).height < 100
    #[arg(long = "assert")]
    assertions: Vec<String>,

    /// Override URL (instead of discovering server)
    ///
    /// Useful when connecting to a specific server or non-standard configuration
    #[arg(long)]
    url: Option<String>,
}

/// CLI argument for --measure that supports "auto" as default missing value
#[derive(Debug, Clone, Copy, clap::ValueEnum)]
enum MeasurePresetArg {
    /// Auto-select based on target type (route → site, path → document)
    Auto,
    /// Document content selectors
    Document,
    /// Site chrome selectors
    Site,
    /// Both document and site selectors
    All,
    /// Header and top-bar selectors
    Header,
    /// Navigation and breadcrumb selectors
    Nav,
    /// Main content selectors
    Main,
    /// Footer selectors
    Footer,
    /// Combined theme review selectors
    Theme,
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
enum ScreenshotResizeModeArg {
    Never,
    Auto,
    Optimize,
}

pub static CLI_AFTER_LONG_HELP: &str = cstr!(
    r#"<bold><b>Examples</b></bold>
  <dim># Snap site root (default route /)</dim>
  <b>stencila snap</> <c>--shot</> <g>homepage.png</>

  <dim># Extract resolved theme token values</dim>
  <b>stencila snap</> <c>--tokens</>

  <dim># Extract color palette</dim>
  <b>stencila snap</> <c>--palette</>

  <dim># Snap a specific site route with site chrome measurements</dim>
  <b>stencila snap</> <g>/docs/guide/</> <c>--measure</> <g>site</>

  <dim># Measure theme-critical regions with grouped color tokens</dim>
  <b>stencila snap</> <c>--measure</> <g>theme</> <c>--tokens</> <c>--token-prefix</> <g>color</>

  <dim># Snap a document file directly</dim>
  <b>stencila snap</> <g>./my-doc.md</> <c>--shot</> <g>doc.png</>

  <dim># Measure at multiple viewports in one call</dim>
  <b>stencila snap</> <c>--devices</> <g>mobile,tablet,laptop</> <c>--measure</>

  <dim># Assert site chrome properties</dim>
  <b>stencila snap</> <c>--assert</> <y>"exists(stencila-logo)==true"</>

  <dim># Full page dark mode screenshot</dim>
  <b>stencila snap</> <c>--dark</> <c>--full</> <c>--shot</> <g>dark-full.png</>

  <dim># Combined: tokens + palette + measurements for theme review</dim>
  <b>stencila snap</> <c>--tokens</> <c>--palette</> <c>--measure</> <g>all</>

  <dim># Verify theme token on header</dim>
  <b>stencila snap</> <c>--assert</> <y>"css(stencila-layout > header).backgroundColorHex==#1a1a2e"</>

  <dim># Capture mobile viewport of specific element</dim>
  <b>stencila snap</> <c>--device</> <g>mobile</> <c>--selector</> <y>"stencila-article [slot=title]"</> <c>--shot</> <g>mobile.png</>

  <dim># Optimize screenshot size for lower image payload cost</dim>
  <b>stencila snap</> <c>--shot</> <g>page.png</> <c>--resize</> <g>optimize</> <c>--max-image-dimension</> <g>4096</>
"#
);

impl Cli {
    pub async fn run(self) -> Result<()> {
        // Determine effective color scheme from flags
        let color_scheme = if self.light {
            ColorScheme::Light
        } else if self.dark {
            ColorScheme::Dark
        } else {
            ColorScheme::System
        };

        // Convert CLI args to SnapOptions
        let viewport = if self.print {
            // Use A4 dimensions for print preview (210mm x 297mm at 96 DPI = 794px x 1123px)
            Some(ViewportConfig {
                width: 794,
                height: 1123,
                dpr: 1.0,
                color_scheme: ColorScheme::System,
            })
        } else if let (Some(width), Some(height)) = (self.width, self.height) {
            Some(ViewportConfig {
                width,
                height,
                dpr: self.dpr.unwrap_or(1.0),
                color_scheme,
            })
        } else {
            None
        };

        // Only override color scheme if it's not the default "system" or if custom viewport was specified
        let color_scheme = if viewport.is_some() || color_scheme != ColorScheme::System {
            Some(color_scheme)
        } else {
            None
        };

        // Convert --measure arg to MeasureMode
        let measure = match self.measure {
            None => MeasureMode::Off,
            Some(MeasurePresetArg::Auto) => MeasureMode::Auto,
            Some(MeasurePresetArg::Document) => MeasureMode::Preset(MeasurePreset::Document),
            Some(MeasurePresetArg::Site) => MeasureMode::Preset(MeasurePreset::Site),
            Some(MeasurePresetArg::All) => MeasureMode::Preset(MeasurePreset::All),
            Some(MeasurePresetArg::Header) => MeasureMode::Preset(MeasurePreset::Header),
            Some(MeasurePresetArg::Nav) => MeasureMode::Preset(MeasurePreset::Nav),
            Some(MeasurePresetArg::Main) => MeasureMode::Preset(MeasurePreset::Main),
            Some(MeasurePresetArg::Footer) => MeasureMode::Preset(MeasurePreset::Footer),
            Some(MeasurePresetArg::Theme) => MeasureMode::Preset(MeasurePreset::Theme),
        };

        // Save shot path before moving fields into SnapOptions
        let shot_path = self.shot.clone();

        let options = SnapOptions {
            route_or_path: self.route_or_path,
            url: self.url,
            screenshot: shot_path.is_some(),
            selector: self.selector,
            full_page: self.full,
            device: self.device,
            devices: self.devices,
            viewport,
            color_scheme,
            print_media: self.print,
            wait_config: WaitConfig {
                wait_until: self.wait_until,
                wait_for: self.wait_for,
                delay: self.delay,
            },
            measure,
            tokens: self.tokens,
            token_prefixes: self.token_prefix,
            palette: self.palette,
            assertions: self.assertions,
            screenshot_resize: ScreenshotResizePolicy {
                mode: match self.resize {
                    ScreenshotResizeModeArg::Never => ScreenshotResizeMode::Never,
                    ScreenshotResizeModeArg::Auto => ScreenshotResizeMode::Auto,
                    ScreenshotResizeModeArg::Optimize => ScreenshotResizeMode::Optimize,
                },
                max_dimension: self.max_image_dimension,
            },
        };

        // Call snap crate
        let result = snap(options).await?;

        // Write primary screenshot to disk if --shot was specified
        if let Some(shot_path) = &shot_path
            && let Some(ref screenshot_data) = result.screenshot
        {
            if let Some(dir) = shot_path.parent() {
                tokio::fs::create_dir_all(dir).await?;
            }
            tokio::fs::write(shot_path, screenshot_data).await?;
        }

        // Write per-device screenshots to disk if --shot and --devices were specified
        if let Some(shot_path) = &shot_path
            && let Some(ref devices) = result.devices
        {
            let stem = shot_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("snap");
            let ext = shot_path
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("png");
            for (device_name, device_result) in devices {
                if let Some(ref data) = device_result.screenshot {
                    let device_path = shot_path
                        .parent()
                        .unwrap_or(std::path::Path::new("."))
                        .join(format!("{stem}-{device_name}.{ext}"));
                    if let Some(dir) = device_path.parent() {
                        tokio::fs::create_dir_all(dir).await?;
                    }
                    tokio::fs::write(&device_path, data).await?;
                }
            }
        }

        // Output JSON to stdout
        Code::new(Format::Json, &result.to_json()?).to_stdout();

        // Set exit code based on result
        if !result.ok {
            if !result.assertions.passed {
                exit(ASSERTION_FAILED);
            }
            exit(BROWSER_FAILURE);
        }

        Ok(())
    }
}
