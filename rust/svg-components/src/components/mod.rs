mod angle;
mod arrow;
mod badge;
mod brace;
mod bracket;
mod callout;
mod compass;
mod crosshair;
mod dimension;
mod halo;
mod marker;
mod roi;
mod scale_bar;
mod spotlight;

use std::collections::HashMap;

use crate::anchors::{Anchor, resolve_anchor_ref};
use crate::compile::xml_escape;
use crate::diagnostics::CompilationMessage;

/// Parsed attributes from an `s:` element.
pub type Attrs = HashMap<String, String>;

#[derive(Clone, Copy)]
struct VectorMetrics {
    dx: f64,
    dy: f64,
    len: f64,
    nx: f64,
    ny: f64,
}

fn vector_metrics(x1: f64, y1: f64, x2: f64, y2: f64) -> VectorMetrics {
    let dx = x2 - x1;
    let dy = y2 - y1;
    let len = (dx * dx + dy * dy).sqrt();
    let (nx, ny) = if len > 0.0 {
        (-dy / len, dx / len)
    } else {
        (0.0, -1.0)
    };

    VectorMetrics {
        dx,
        dy,
        len,
        nx,
        ny,
    }
}

fn normal_for_side(metrics: &VectorMetrics, side: &str) -> (f64, f64) {
    match side {
        "above" | "right" => (-metrics.nx, -metrics.ny),
        _ => (metrics.nx, metrics.ny),
    }
}

fn quad_control_point(x1: f64, y1: f64, x2: f64, y2: f64) -> QuadControlPoint {
    let metrics = vector_metrics(x1, y1, x2, y2);
    let offset = metrics.len * 0.25;

    QuadControlPoint {
        cx: f64::midpoint(x1, x2) + metrics.nx * offset,
        cy: f64::midpoint(y1, y2) + metrics.ny * offset,
    }
}

fn cubic_control_points(x1: f64, y1: f64, x2: f64, y2: f64) -> CubicControlPoints {
    let metrics = vector_metrics(x1, y1, x2, y2);
    let offset = metrics.len * 0.25;

    CubicControlPoints {
        cx1: x1 + metrics.dx / 3.0 + metrics.nx * offset,
        cy1: y1 + metrics.dy / 3.0 + metrics.ny * offset,
        cx2: x1 + 2.0 * metrics.dx / 3.0 - metrics.nx * offset,
        cy2: y1 + 2.0 * metrics.dy / 3.0 - metrics.ny * offset,
    }
}

#[derive(Clone, Copy)]
struct QuadControlPoint {
    cx: f64,
    cy: f64,
}

#[derive(Clone, Copy)]
struct CubicControlPoints {
    cx1: f64,
    cy1: f64,
    cx2: f64,
    cy2: f64,
}

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
        "angle" => Some(angle::expand(attrs, ctx)),
        "arrow" => Some(arrow::expand(attrs, ctx)),
        "badge" => Some(badge::expand(attrs, ctx)),
        "brace" => Some(brace::expand(attrs, ctx)),
        "bracket" => Some(bracket::expand(attrs, ctx)),
        "callout" => Some(callout::expand(attrs, ctx)),
        "compass" => Some(compass::expand(attrs, ctx)),
        "crosshair" => Some(crosshair::expand(attrs, ctx)),
        "dimension" => Some(dimension::expand(attrs, ctx)),
        "halo" => Some(halo::expand(attrs, ctx)),
        "marker" => Some(marker::expand(attrs, ctx)),
        "roi-ellipse" => Some(roi::expand_ellipse(attrs, ctx)),
        "roi-polygon" => Some(roi::expand_polygon(attrs, ctx)),
        "roi-rect" => Some(roi::expand_rect(attrs, ctx)),
        "scale-bar" => Some(scale_bar::expand(attrs, ctx)),
        "spotlight" => Some(spotlight::expand(attrs, ctx)),
        _ => None,
    }
}

// --- Attribute parsing helpers ---

/// Get a string attribute with a default value.
fn attr_str<'a>(attrs: &'a Attrs, key: &str, default: &'a str) -> &'a str {
    attrs.get(key).map_or(default, std::string::String::as_str)
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
fn svg_text(
    x: f64,
    y: f64,
    text: &str,
    anchor: &str,
    font_size: u32,
    fill: &str,
    extra: &str,
) -> String {
    format!(
        r#"<text x="{}" y="{}" text-anchor="{anchor}" font-size="{font_size}" fill="{fill}"{extra}>{}</text>"#,
        fmt_coord(x),
        fmt_coord(y),
        xml_escape(text)
    )
}

/// Generate an SVG `<line>` element.
fn svg_line(x1: f64, y1: f64, x2: f64, y2: f64, stroke: &str, extra: &str) -> String {
    format!(
        r#"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="{stroke}"{extra}/>"#,
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
    stroke: &'a str,
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
                r#"<polyline points="{},{} {},{} {},{}" fill="none" stroke="{}"{}{}{}/>"#,
                fmt_coord(x1),
                fmt_coord(y1),
                fmt_coord(mx),
                fmt_coord(my),
                fmt_coord(x2),
                fmt_coord(y2),
                opts.stroke,
                opts.marker_start,
                opts.marker_end,
                opts.pass
            )
        }
        "quad" => {
            let QuadControlPoint { cx, cy } = quad_control_point(x1, y1, x2, y2);
            format!(
                r#"<path d="M {},{} Q {},{} {},{}" fill="none" stroke="{}"{}{}{}/>"#,
                fmt_coord(x1),
                fmt_coord(y1),
                fmt_coord(cx),
                fmt_coord(cy),
                fmt_coord(x2),
                fmt_coord(y2),
                opts.stroke,
                opts.marker_start,
                opts.marker_end,
                opts.pass
            )
        }
        "cubic" => {
            let CubicControlPoints { cx1, cy1, cx2, cy2 } = cubic_control_points(x1, y1, x2, y2);
            format!(
                r#"<path d="M {},{} C {},{} {},{} {},{}" fill="none" stroke="{}"{}{}{}/>"#,
                fmt_coord(x1),
                fmt_coord(y1),
                fmt_coord(cx1),
                fmt_coord(cy1),
                fmt_coord(cx2),
                fmt_coord(cy2),
                fmt_coord(x2),
                fmt_coord(y2),
                opts.stroke,
                opts.marker_start,
                opts.marker_end,
                opts.pass
            )
        }
        _ => {
            // straight (default)
            format!(
                r#"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="{}"{}{}{}/>"#,
                fmt_coord(x1),
                fmt_coord(y1),
                fmt_coord(x2),
                fmt_coord(y2),
                opts.stroke,
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

/// Resolve the stroke color: explicit `stroke` attr, then `color` shorthand, then `currentColor`.
fn resolve_stroke(attrs: &Attrs) -> &str {
    attrs
        .get("stroke")
        .or_else(|| attrs.get("color"))
        .map_or("currentColor", std::string::String::as_str)
}

/// Resolve the fill color: explicit `fill` attr, then `color` shorthand, then the given default.
fn resolve_fill<'a>(attrs: &'a Attrs, default: &'a str) -> &'a str {
    attrs
        .get("fill")
        .or_else(|| attrs.get("color"))
        .map_or(default, std::string::String::as_str)
}

/// Resolve the text color: explicit `text` attr, then `color` shorthand, then `currentColor`.
fn resolve_text(attrs: &Attrs) -> &str {
    attrs
        .get("text")
        .or_else(|| attrs.get("color"))
        .map_or("currentColor", std::string::String::as_str)
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
        "r",
        "gap",
        "ring",
        "points",
        "depth",
        "at",
        "from-x",
        "from-y",
        "vertex",
        "fill",
        "stroke",
        "color",
        "text",
        "background",
        "opacity",
    ];

    let mut result = String::new();
    let mut keys: Vec<&String> = attrs.keys().collect();
    keys.sort();
    for key in keys {
        let value = &attrs[key];
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

#[cfg(test)]
mod tests {
    use super::*;

    fn attrs(pairs: &[(&str, &str)]) -> Attrs {
        pairs
            .iter()
            .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
            .collect()
    }

    #[test]
    fn resolve_stroke_explicit() {
        assert_eq!(resolve_stroke(&attrs(&[("stroke", "red")])), "red");
    }

    #[test]
    fn resolve_stroke_color_fallback() {
        assert_eq!(resolve_stroke(&attrs(&[("color", "blue")])), "blue");
    }

    #[test]
    fn resolve_stroke_explicit_over_color() {
        assert_eq!(
            resolve_stroke(&attrs(&[("stroke", "red"), ("color", "blue")])),
            "red"
        );
    }

    #[test]
    fn resolve_stroke_default() {
        assert_eq!(resolve_stroke(&attrs(&[])), "currentColor");
    }

    #[test]
    fn svg_line_uses_custom_stroke() {
        let line = svg_line(0.0, 0.0, 10.0, 10.0, "red", "");
        assert!(line.contains(r#"stroke="red""#));
        assert!(!line.contains("currentColor"));
    }

    #[test]
    fn svg_line_with_extra_attrs() {
        let line = svg_line(0.0, 0.0, 10.0, 10.0, "currentColor", r#" stroke-width="2""#);
        assert!(line.contains(r#"stroke="currentColor""#));
        assert!(line.contains(r#"stroke-width="2""#));
    }

    #[test]
    fn resolve_fill_explicit() {
        assert_eq!(resolve_fill(&attrs(&[("fill", "red")]), "white"), "red");
    }

    #[test]
    fn resolve_fill_color_fallback() {
        assert_eq!(resolve_fill(&attrs(&[("color", "blue")]), "white"), "blue");
    }

    #[test]
    fn resolve_fill_explicit_over_color() {
        assert_eq!(
            resolve_fill(&attrs(&[("fill", "red"), ("color", "blue")]), "white"),
            "red"
        );
    }

    #[test]
    fn resolve_fill_default() {
        assert_eq!(resolve_fill(&attrs(&[]), "white"), "white");
        assert_eq!(resolve_fill(&attrs(&[]), "currentColor"), "currentColor");
    }

    #[test]
    fn resolve_text_explicit() {
        assert_eq!(resolve_text(&attrs(&[("text", "red")])), "red");
    }

    #[test]
    fn resolve_text_color_fallback() {
        assert_eq!(resolve_text(&attrs(&[("color", "blue")])), "blue");
    }

    #[test]
    fn resolve_text_explicit_over_color() {
        assert_eq!(
            resolve_text(&attrs(&[("text", "red"), ("color", "blue")])),
            "red"
        );
    }

    #[test]
    fn resolve_text_default() {
        assert_eq!(resolve_text(&attrs(&[])), "currentColor");
    }

    #[test]
    fn pass_through_excludes_fill_and_stroke() {
        let a = attrs(&[("fill", "red"), ("stroke", "blue"), ("stroke-width", "2")]);
        let pass = pass_through_attrs(&a);
        assert!(!pass.contains(r#"fill="red""#));
        assert!(!pass.contains(r#"stroke="blue""#));
        assert!(pass.contains(r#"stroke-width="2""#));
    }
}
