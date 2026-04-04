mod arrow;
mod badge;
mod brace;
mod callout;
mod compass;
mod dimension;
mod marker;
mod roi;
mod scale_bar;

use std::collections::HashMap;

use crate::anchors::{Anchor, resolve_anchor_ref};
use crate::compile::xml_escape;
use crate::diagnostics::CompilationMessage;

/// Parsed attributes from an `s:` element.
pub type Attrs = HashMap<String, String>;

/// The context available to a component expander.
pub struct ComponentContext<'a> {
    pub anchors: &'a HashMap<String, Anchor>,
    pub messages: &'a mut Vec<CompilationMessage>,
}

/// Expand a Stencila SVG component into standard SVG.
///
/// Returns `Some(String)` if the element was recognized and expanded,
/// or `None` if the element is not a known component (which will generate a warning).
pub fn expand_component(name: &str, attrs: &Attrs, ctx: &mut ComponentContext) -> Option<String> {
    match name {
        "arrow" => Some(arrow::expand(attrs, ctx)),
        "callout" => Some(callout::expand(attrs, ctx)),
        "badge" => Some(badge::expand(attrs, ctx)),
        "scale-bar" => Some(scale_bar::expand(attrs, ctx)),
        "dimension" => Some(dimension::expand(attrs, ctx)),
        "brace" => Some(brace::expand(attrs, ctx)),
        "roi-rect" => Some(roi::expand_rect(attrs, ctx)),
        "roi-ellipse" => Some(roi::expand_ellipse(attrs, ctx)),
        "marker" => Some(marker::expand(attrs, ctx)),
        "compass" => Some(compass::expand(attrs, ctx)),
        _ => None,
    }
}

// --- Attribute parsing helpers ---

/// Get a string attribute with a default value.
fn attr_str<'a>(attrs: &'a Attrs, key: &str, default: &'a str) -> &'a str {
    attrs.get(key).map(|s| s.as_str()).unwrap_or(default)
}

/// Parse a float attribute, returning `None` if missing or unparseable.
fn attr_f64(attrs: &Attrs, key: &str) -> Option<f64> {
    attrs.get(key).and_then(|v| v.parse::<f64>().ok())
}

/// Parse a float attribute with a default value.
fn attr_f64_or(attrs: &Attrs, key: &str, default: f64) -> f64 {
    attr_f64(attrs, key).unwrap_or(default)
}

// --- SVG element helpers ---

/// Generate an SVG `<text>` element.
fn svg_text(x: f64, y: f64, text: &str, anchor: &str, font_size: u32, extra: &str) -> String {
    format!(
        r#"<text x="{}" y="{}" text-anchor="{anchor}" font-size="{font_size}"{extra}>{}</text>"#,
        fmt_coord(x),
        fmt_coord(y),
        xml_escape(text)
    )
}

/// Generate an SVG `<line>` element.
fn svg_line(x1: f64, y1: f64, x2: f64, y2: f64, extra: &str) -> String {
    format!(
        r#"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="currentColor"{extra}/>"#,
        fmt_coord(x1),
        fmt_coord(y1),
        fmt_coord(x2),
        fmt_coord(y2),
    )
}

// --- Connector/curve helper ---

/// Options for rendering a connector path between two points.
struct ConnectorOpts<'a> {
    curve: &'a str,
    corner: &'a str,
    marker_start: &'a str,
    marker_end: &'a str,
    pass: &'a str,
}

/// Generate a connector path (line, elbow, quad, or cubic curve) between two points.
fn connector_svg(x1: f64, y1: f64, x2: f64, y2: f64, opts: &ConnectorOpts) -> String {
    match opts.curve {
        "elbow" => {
            let (mx, my) = if opts.corner == "vertical-first" {
                (x1, y2)
            } else {
                (x2, y1)
            };
            format!(
                r#"<polyline points="{},{} {},{} {},{}" fill="none" stroke="currentColor"{}{}{}/>"#,
                fmt_coord(x1),
                fmt_coord(y1),
                fmt_coord(mx),
                fmt_coord(my),
                fmt_coord(x2),
                fmt_coord(y2),
                opts.marker_start,
                opts.marker_end,
                opts.pass
            )
        }
        "quad" => {
            // Offset the control point perpendicular to the chord so the
            // curve visibly bows out. Without this, placing the control
            // point on the chord produces a straight line.
            let dx = x2 - x1;
            let dy = y2 - y1;
            let len = (dx * dx + dy * dy).sqrt();
            let offset = len * 0.25;
            let (nx, ny) = if len > 0.0 {
                (-dy / len, dx / len)
            } else {
                (0.0, -1.0)
            };
            let cx = (x1 + x2) / 2.0 + nx * offset;
            let cy = (y1 + y2) / 2.0 + ny * offset;
            format!(
                r#"<path d="M {},{} Q {},{} {},{}" fill="none" stroke="currentColor"{}{}{}/>"#,
                fmt_coord(x1),
                fmt_coord(y1),
                fmt_coord(cx),
                fmt_coord(cy),
                fmt_coord(x2),
                fmt_coord(y2),
                opts.marker_start,
                opts.marker_end,
                opts.pass
            )
        }
        "cubic" => {
            // S-curve: offset control points perpendicular to the chord so
            // the curve has proper tangent angles at both endpoints.
            let dx = x2 - x1;
            let dy = y2 - y1;
            let len = (dx * dx + dy * dy).sqrt();
            let offset = len * 0.25;
            let (nx, ny) = if len > 0.0 {
                (-dy / len, dx / len)
            } else {
                (0.0, -1.0)
            };
            // First control point: 1/3 along the chord, offset to one side
            let cx1 = x1 + dx / 3.0 + nx * offset;
            let cy1 = y1 + dy / 3.0 + ny * offset;
            // Second control point: 2/3 along the chord, offset to the other side
            let cx2 = x1 + 2.0 * dx / 3.0 - nx * offset;
            let cy2 = y1 + 2.0 * dy / 3.0 - ny * offset;
            format!(
                r#"<path d="M {},{} C {},{} {},{} {},{}" fill="none" stroke="currentColor"{}{}{}/>"#,
                fmt_coord(x1),
                fmt_coord(y1),
                fmt_coord(cx1),
                fmt_coord(cy1),
                fmt_coord(cx2),
                fmt_coord(cy2),
                fmt_coord(x2),
                fmt_coord(y2),
                opts.marker_start,
                opts.marker_end,
                opts.pass
            )
        }
        _ => {
            // straight (default)
            format!(
                r#"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="currentColor"{}{}{}/>"#,
                fmt_coord(x1),
                fmt_coord(y1),
                fmt_coord(x2),
                fmt_coord(y2),
                opts.marker_start,
                opts.marker_end,
                opts.pass
            )
        }
    }
}

/// Build a marker attribute string for a given tip direction and style.
fn marker_attrs(tip: &str, tip_style: &str) -> (String, String) {
    let marker_start = match tip {
        "start" | "both" => format!(r#" marker-start="url(#{tip_style})""#),
        _ => String::new(),
    };
    let marker_end = match tip {
        "end" | "both" => format!(r#" marker-end="url(#{tip_style})""#),
        _ => String::new(),
    };
    (marker_start, marker_end)
}

/// Helper to resolve a position from either direct coordinates or anchor references.
///
/// Checks for `{prefix}` as `x`/`y` direct coordinates, or `{prefix}` as an anchor
/// reference with optional `dx`/`dy` offsets.
fn resolve_position(
    attrs: &Attrs,
    x_attr: &str,
    y_attr: &str,
    from_attr: Option<&str>,
    dx_attr: &str,
    dy_attr: &str,
    anchors: &HashMap<String, Anchor>,
) -> Option<(f64, f64)> {
    let x = attr_f64(attrs, x_attr);
    let y = attr_f64(attrs, y_attr);

    if let (Some(x), Some(y)) = (x, y) {
        let dx = attr_f64_or(attrs, dx_attr, 0.0);
        let dy = attr_f64_or(attrs, dy_attr, 0.0);
        return Some((x + dx, y + dy));
    }

    if let Some(from_key) = from_attr
        && let Some(anchor_ref) = attrs.get(from_key)
    {
        let dx = attr_f64_or(attrs, dx_attr, 0.0);
        let dy = attr_f64_or(attrs, dy_attr, 0.0);
        return resolve_anchor_ref(anchor_ref, anchors, dx, dy);
    }

    None
}

/// Helper to resolve a target position (to-x/to-y or to anchor ref).
fn resolve_target(attrs: &Attrs, anchors: &HashMap<String, Anchor>) -> Option<(f64, f64)> {
    let x = attr_f64(attrs, "to-x");
    let y = attr_f64(attrs, "to-y");

    if let (Some(x), Some(y)) = (x, y) {
        return Some((x, y));
    }

    if let Some(anchor_ref) = attrs.get("to") {
        return resolve_anchor_ref(anchor_ref, anchors, 0.0, 0.0);
    }

    None
}

/// Collect SVG presentation attributes to pass through.
fn pass_through_attrs(attrs: &Attrs) -> String {
    // Known s: component attributes that should NOT be passed through
    const COMPONENT_ATTRS: &[&str] = &[
        "x",
        "y",
        "dx",
        "dy",
        "from",
        "to",
        "to-x",
        "to-y",
        "label",
        "label-position",
        "label-angle",
        "shape",
        "symbol",
        "variant",
        "stroke-style",
        "tip",
        "tip-style",
        "curve",
        "corner",
        "side",
        "width",
        "height",
        "length",
        "size",
        "bulge",
        "rx",
        "ry",
        "cx",
        "cy",
        "axes",
        "id",
    ];

    let mut result = String::new();
    for (key, value) in attrs {
        if !COMPONENT_ATTRS.contains(&key.as_str()) {
            result.push(' ');
            result.push_str(key);
            result.push_str("=\"");
            result.push_str(&xml_escape(value));
            result.push('"');
        }
    }
    result
}

/// Format a coordinate value, omitting trailing decimal point for whole numbers.
fn fmt_coord(v: f64) -> String {
    if v.fract() == 0.0 {
        format!("{v:.0}")
    } else {
        format!("{v}")
    }
}
