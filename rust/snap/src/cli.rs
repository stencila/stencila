//! CLI wrapper for snap command

use std::{path::PathBuf, process::exit};

use clap::Parser;
use eyre::Result;

use stencila_cli_utils::{Code, ToStdout, color_print::cstr};
use stencila_format::Format;

use crate::{SnapOptions, snap};

use super::{
    browser::{ColorScheme, WaitConfig, WaitUntil},
    devices::{DevicePreset, ViewportConfig},
};

const ASSERTION_FAILED: i32 = 12;
const SELECTOR_NOT_FOUND: i32 = 20;
const TIMEOUT: i32 = 30;
const BROWSER_FAILURE: i32 = 40;

/// Capture screenshots and measurements of documents served by Stencila
///
/// The `snap` command allows programmatic screenshotting and measurement of
/// documents served by Stencila. It can be used to:
///
/// - Iterate on themes and styled elements and verify changes
/// - Capture screenshots for documentation or CI
/// - Assert computed CSS properties and layout metrics
/// - Measure page elements for automated testing
#[derive(Debug, Parser)]
#[command(after_long_help = CLI_AFTER_LONG_HELP)]
pub struct Cli {
    /// Path to document or directory
    ///
    /// If not specified, will use the current directory. The path should be
    /// within the directory being served by a running Stencila server.
    path: Option<PathBuf>,

    /// Output screenshot path (.png)
    ///
    /// If specified, a screenshot will be captured. If not specified, only
    /// measurements and assertions will be performed (no screenshot).
    output: Option<PathBuf>,

    /// CSS selector to capture or measure
    ///
    /// If specified, screenshots will be cropped to this element and
    /// measurements will focus on it.
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
    /// Use a predefined viewport configuration: laptop, desktop, iphone-15,
    /// ipad, ipad-landscape
    #[arg(long, value_enum)]
    device: Option<DevicePreset>,

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
    #[arg(long)]
    measure: bool,

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

pub static CLI_AFTER_LONG_HELP: &str = cstr!(
    r#"<bold><b>Examples</b></bold>
  <dim># Start server in background</dim>
  <b>stencila serve</> <c>--sync</> <g>in</> &

  <dim># Capture viewport screenshot (default)</dim>
  <b>stencila snap</> <g>snaps/viewport.png</>

  <dim># Capture full scrollable page</dim>
  <b>stencila snap</> <c>--full</> <g>snaps/full.png</>

  <dim># Verify computed padding for title</dim>
  <b>stencila snap</> <c>--assert</> <y>"css([slot=title]).paddingTop>=24px"</>

  <dim># Capture mobile viewport of specific element</dim>
  <b>stencila snap</> <c>--device</> <g>mobile</> <c>--selector</> <y>"stencila-article [slot=title]"</> <g>snaps/mobile.png</>

  <dim># Capture full mobile page</dim>
  <b>stencila snap</> <c>--device</> <g>mobile</> <c>--full</> <g>snaps/mobile-full.png</>

  <dim># Force light or dark mode</dim>
  <b>stencila snap</> <c>--light</> <g>snaps/light.png</>
  <b>stencila snap</> <c>--dark</> <g>snaps/dark.png</>

  <dim># Preview with PDF/print styles (A4 width)</dim>
  <b>stencila snap</> <c>--print</> <g>snaps/print-preview.png</>

  <dim># Multiple assertions without screenshot</dim>
  <b>stencila snap</> \
    <c>--assert</> <y>"css([slot=title]).fontSize>=28px"</> \
    <c>--assert</> <y>"count(section)==5"</> \
    <c>--measure</>

  <dim># Use custom viewport and wait conditions</dim>
  <b>stencila snap</> \
    <c>--width</> <g>1920</> <c>--height</> <g>1080</> \
    <c>--wait-until</> <g>networkidle</> \
    <c>--delay</> <g>500</> \
    <g>snaps/desktop.png</>

  <dim># Capture specific document path</dim>
  <b>stencila snap</> <g>docs/guide.md</> <g>snaps/guide.png</>
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

        let options = SnapOptions {
            path: self.path,
            url: self.url,
            output: self.output,
            selector: self.selector,
            full_page: self.full,
            device: self.device,
            viewport,
            color_scheme,
            print_media: self.print,
            wait_config: WaitConfig {
                wait_until: self.wait_until,
                wait_for: self.wait_for,
                delay: self.delay,
            },
            measure: self.measure,
            assertions: self.assertions,
        };

        // Call snap crate
        let result = snap(options).await?;

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
