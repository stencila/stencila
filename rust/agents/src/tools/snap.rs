//! `snap` tool: capture screenshots and measurements of Stencila documents and sites.
//!
//! Calls the `stencila-snap` crate directly. The snap crate no longer depends
//! on `stencila-server` (it reads server info files from disk), which breaks
//! the cyclic dependency: agents → snap → server → attractor → agents.

use serde_json::{Value, json};
use stencila_models3::types::tool::ToolDefinition;
use stencila_snap::{
    ColorScheme, DevicePreset, MeasureMode, MeasurePreset, SnapOptions, ViewportConfig, WaitConfig,
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
                    "description": "Route on the running Stencila server (e.g. \"/\" or \"/docs/guide/\"). Defaults to \"/\"."
                },
                "screenshot": {
                    "type": "boolean",
                    "description": "Whether to capture a screenshot. Defaults to true.",
                    "default": true
                },
                "selector": {
                    "type": "string",
                    "description": "CSS selector to capture or measure a specific element."
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
                "devices": {
                    "type": "array",
                    "items": {
                        "type": "string",
                        "enum": ["laptop", "desktop", "mobile", "tablet", "tablet-landscape"]
                    },
                    "description": "Multiple device presets for batch measurement."
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
                    "enum": ["auto", "document", "site", "all"]
                },
                "tokens": {
                    "type": "boolean",
                    "description": "Extract resolved CSS custom property (theme token) values.",
                    "default": false
                },
                "palette": {
                    "type": "boolean",
                    "description": "Extract the page's color palette.",
                    "default": false
                },
                "assert": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Assertion expressions to evaluate (e.g. \"css(.title).fontSize>=28px\")."
                },
                "wait_for": {
                    "type": "string",
                    "description": "Wait for a CSS selector to exist before capturing."
                },
                "delay": {
                    "type": "integer",
                    "description": "Additional delay in milliseconds after page is ready."
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
        .unwrap_or(true);
    let full_page = args
        .get("full_page")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let dark = args.get("dark").and_then(Value::as_bool).unwrap_or(false);
    let light = args.get("light").and_then(Value::as_bool).unwrap_or(false);
    let print_media = args.get("print").and_then(Value::as_bool).unwrap_or(false);
    let tokens = args.get("tokens").and_then(Value::as_bool).unwrap_or(false);
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
    } else if let (Some(w), Some(h)) = (
        args.get("width").and_then(Value::as_u64),
        args.get("height").and_then(Value::as_u64),
    ) {
        Some(ViewportConfig {
            width: w as u32,
            height: h as u32,
            dpr: 1.0,
            color_scheme,
        })
    } else {
        None
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
        _ => MeasureMode::Off,
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

    let devices: Option<Vec<DevicePreset>> = args
        .get("devices")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .map(|v| {
                    let s = v.as_str().ok_or_else(|| AgentError::ValidationError {
                        reason: "each entry in 'devices' must be a string".into(),
                    })?;
                    parse_device(s)
                })
                .collect::<AgentResult<Vec<_>>>()
        })
        .transpose()?;

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
        devices,
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
        palette,
        assertions,
    };

    let result = stencila_snap::snap(options)
        .await
        .map_err(|error| AgentError::Io {
            message: format!("snap failed: {error}"),
        })?;

    let screenshot_bytes = result.screenshot.clone();

    let json_text = result.to_json().map_err(|error| AgentError::Io {
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
