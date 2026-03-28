//! `snap` tool: capture screenshots and measurements of Stencila documents and sites.
//!
//! Calls the `stencila-snap` crate directly. The snap crate no longer depends
//! on `stencila-server` (it reads server info files from disk), which breaks
//! the cyclic dependency: agents → snap → server → attractor → agents.

use serde_json::{Value, json};
use stencila_models3::types::tool::ToolDefinition;
use stencila_snap::{
    ColorScheme, DevicePreset, MeasureMode, MeasurePreset, ScreenshotResizeMode,
    ScreenshotResizePolicy, SnapOptions, ViewportConfig, WaitConfig,
};

use crate::error::{AgentError, AgentResult};
use crate::registry::{ToolExecutorFn, ToolOutput};

pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "snap".into(),
        description: "Capture a screenshot and/or collect measurements of a page served by \
            Stencila. Returns JSON measurement data as text and, when screenshot is enabled, \
            the PNG image for visual inspection."
            .into(),
        parameters: json!({
            "type": "object",
            "properties": {
                "route": {
                    "type": "string",
                    "description": "Route on the running Stencila server or a document path on disk. Defaults to \"/\"."
                },
                "screenshot": {
                    "type": "boolean",
                    "description": "Whether to capture a screenshot. Defaults to false.",
                    "default": false
                },
                "selector": {
                    "type": "string",
                    "description": "CSS selector to capture or measure a specific element. Takes precedence over full-page capture."
                },
                "full_page": {
                    "type": "boolean",
                    "description": "Capture the full scrollable page instead of just the viewport.",
                    "default": false
                },
                "device": {
                    "type": "string",
                    "description": "Device preset for viewport dimensions.",
                    "enum": ["laptop", "desktop", "mobile", "tablet", "tablet-landscape"]
                },
                "width": {
                    "type": "integer",
                    "description": "Custom viewport width in pixels."
                },
                "height": {
                    "type": "integer",
                    "description": "Custom viewport height in pixels."
                },
                "dark": {
                    "type": "boolean",
                    "description": "Force dark color scheme. Mutually exclusive with 'light' and 'print'.",
                    "default": false
                },
                "light": {
                    "type": "boolean",
                    "description": "Force light color scheme. Mutually exclusive with 'dark' and 'print'.",
                    "default": false
                },
                "print": {
                    "type": "boolean",
                    "description": "Emulate print media (A4 dimensions). Incompatible with 'dark' and 'light'.",
                    "default": false
                },
                "measure": {
                    "type": "string",
                    "description": "Measurement preset determining which selectors to measure.",
                    "enum": ["auto", "document", "site", "all", "header", "nav", "main", "footer", "theme"]
                },
                "tokens": {
                    "type": "boolean",
                    "description": "Extract resolved CSS custom property (theme token) values, grouped by token family.",
                    "default": false
                },
                "token_prefix": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Optional CSS custom property prefixes to filter token output, e.g. [\"color\", \"font\"]. Requires 'tokens'."
                },
                "palette": {
                    "type": "boolean",
                    "description": "Extract the page's color palette.",
                    "default": false
                },
                "assert": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Assertion expressions to evaluate (e.g. \"css(stencila-paragraph).fontSize>=16px\")."
                },
                "wait_for": {
                    "type": "string",
                    "description": "Wait for a CSS selector to exist before capturing."
                },
                "delay": {
                    "type": "integer",
                    "description": "Additional delay in milliseconds after page is ready."
                },
                "resize": {
                    "type": "string",
                    "description": "Screenshot resize mode. 'auto' resizes only when the image exceeds hard provider limits (8000px), 'optimize' downscales more aggressively to reduce payload size, and 'never' preserves the original capture.",
                    "enum": ["never", "auto", "optimize"],
                    "default": "auto"
                },
                "max_image_dimension": {
                    "type": "integer",
                    "description": "Maximum screenshot dimension in pixels after resize. Used with 'auto' (default 8000px) or 'optimize' (default 4096px).",
                    "minimum": 1
                }
            },
            "additionalProperties": false
        }),
        strict: false,
    }
}

pub fn executor() -> ToolExecutorFn {
    Box::new(
        |args: Value, _env: &dyn crate::execution::ExecutionEnvironment| {
            Box::pin(async move { execute(args).await })
        },
    )
}

async fn execute(args: Value) -> AgentResult<ToolOutput> {
    let screenshot = args
        .get("screenshot")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let full_page = args
        .get("full_page")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let dark = args.get("dark").and_then(Value::as_bool).unwrap_or(false);
    let light = args.get("light").and_then(Value::as_bool).unwrap_or(false);
    let print_media = args.get("print").and_then(Value::as_bool).unwrap_or(false);
    let tokens = args.get("tokens").and_then(Value::as_bool).unwrap_or(false);
    let token_prefixes: Vec<String> = args
        .get("token_prefix")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    let palette = args
        .get("palette")
        .and_then(Value::as_bool)
        .unwrap_or(false);

    if dark && light {
        return Err(AgentError::ValidationError {
            reason: "'dark' and 'light' are mutually exclusive".into(),
        });
    }

    if print_media && (dark || light) {
        return Err(AgentError::ValidationError {
            reason: "'print' is incompatible with 'dark' and 'light' \
                     (print mode uses system color scheme)"
                .into(),
        });
    }

    if !tokens && !token_prefixes.is_empty() {
        return Err(AgentError::ValidationError {
            reason: "'token_prefix' requires 'tokens' to be enabled".into(),
        });
    }

    if tokens && token_prefixes.is_empty() {
        return Err(AgentError::ValidationError {
            reason: "'tokens' requires at least one 'token_prefix' to avoid \
                     flooding the context with hundreds of token values"
                .into(),
        });
    }

    let color_scheme = if dark {
        ColorScheme::Dark
    } else if light {
        ColorScheme::Light
    } else {
        ColorScheme::System
    };

    // Build viewport: print → A4, custom w×h, or None (use device preset / default)
    let viewport = if print_media {
        Some(ViewportConfig {
            width: 794,
            height: 1123,
            dpr: 1.0,
            color_scheme: ColorScheme::System,
        })
    } else {
        let custom_width = args.get("width").and_then(Value::as_u64).map(|v| v as u32);
        let custom_height = args.get("height").and_then(Value::as_u64).map(|v| v as u32);
        if let Some(w) = custom_width.or(custom_height.map(|_| 1920)) {
            let h = custom_height.unwrap_or(1080);
            Some(ViewportConfig {
                width: w,
                height: h,
                dpr: 1.0,
                color_scheme,
            })
        } else {
            None
        }
    };

    let effective_color_scheme = if viewport.is_some() || color_scheme != ColorScheme::System {
        Some(color_scheme)
    } else {
        None
    };

    let measure = match args.get("measure").and_then(Value::as_str) {
        Some("auto") => MeasureMode::Auto,
        Some("document") => MeasureMode::Preset(MeasurePreset::Document),
        Some("site") => MeasureMode::Preset(MeasurePreset::Site),
        Some("all") => MeasureMode::Preset(MeasurePreset::All),
        Some("header") => MeasureMode::Preset(MeasurePreset::Header),
        Some("nav") => MeasureMode::Preset(MeasurePreset::Nav),
        Some("main") => MeasureMode::Preset(MeasurePreset::Main),
        Some("footer") => MeasureMode::Preset(MeasurePreset::Footer),
        Some("theme") => MeasureMode::Preset(MeasurePreset::Theme),
        None => MeasureMode::Off,
        Some(other) => {
            return Err(AgentError::ValidationError {
                reason: format!(
                    "unknown measure preset: {other:?}. Valid values: auto, document, site, all, header, nav, main, footer, theme"
                ),
            });
        }
    };

    let assertions: Vec<String> = args
        .get("assert")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    let screenshot_resize_mode = match args.get("resize").and_then(Value::as_str) {
        Some("never") => ScreenshotResizeMode::Never,
        Some("auto") => ScreenshotResizeMode::Auto,
        Some("optimize") => ScreenshotResizeMode::Optimize,
        None => ScreenshotResizeMode::Auto,
        Some(other) => {
            return Err(AgentError::ValidationError {
                reason: format!(
                    "unknown resize mode: {other:?}. Valid values: never, auto, optimize"
                ),
            });
        }
    };

    let options = SnapOptions {
        route_or_path: args.get("route").and_then(Value::as_str).map(String::from),
        url: None,
        screenshot,
        selector: args
            .get("selector")
            .and_then(Value::as_str)
            .map(String::from),
        full_page,
        device: args
            .get("device")
            .and_then(Value::as_str)
            .map(parse_device)
            .transpose()?,
        viewport,
        color_scheme: effective_color_scheme,
        print_media,
        wait_config: WaitConfig {
            wait_for: args
                .get("wait_for")
                .and_then(Value::as_str)
                .map(String::from),
            delay: args.get("delay").and_then(Value::as_u64),
            ..Default::default()
        },
        measure,
        tokens,
        token_prefixes,
        palette,
        assertions,
        screenshot_resize: ScreenshotResizePolicy {
            mode: screenshot_resize_mode,
            max_dimension: args
                .get("max_image_dimension")
                .and_then(Value::as_u64)
                .map(|value| value as u32),
        },
    };

    // The snap crate uses the synchronous `headless_chrome` library internally,
    // so all browser operations (launch, navigate, measure, capture) block the
    // calling thread despite the functions being marked `async`. Offload the
    // entire snap operation to a blocking thread so the tokio runtime stays free
    // (e.g. to drive TUI tick events and spinner animations).
    let handle = tokio::runtime::Handle::current();
    let output = tokio::task::spawn_blocking(move || {
        handle.block_on(async move { stencila_snap::snap(options).await })
    })
    .await
    .map_err(|error| AgentError::Io {
        message: format!("snap task panicked: {error}"),
    })?
    .map_err(|error| AgentError::Io {
        message: format!("snap failed: {error}"),
    })?;

    let screenshot_bytes = output.screenshot.clone();

    let json_text = output.to_json().map_err(|error| AgentError::Io {
        message: format!("failed to serialize snap result: {error}"),
    })?;

    if let Some(data) = screenshot_bytes {
        Ok(ToolOutput::ImageWithText {
            text: json_text,
            data,
            media_type: "image/png".to_string(),
        })
    } else {
        Ok(ToolOutput::Text(json_text))
    }
}

fn parse_device(s: &str) -> AgentResult<DevicePreset> {
    match s {
        "laptop" => Ok(DevicePreset::Laptop),
        "desktop" => Ok(DevicePreset::Desktop),
        "mobile" => Ok(DevicePreset::Mobile),
        "tablet" => Ok(DevicePreset::Tablet),
        "tablet-landscape" => Ok(DevicePreset::TabletLandscape),
        other => Err(AgentError::ValidationError {
            reason: format!(
                "unknown device preset: {other:?}. \
                 Valid values: laptop, desktop, mobile, tablet, tablet-landscape"
            ),
        }),
    }
}
