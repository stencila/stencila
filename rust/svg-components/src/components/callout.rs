use std::fmt::Write;

use super::{
    Attrs, CompilationMessage, ComponentContext, ConnectorOpts, attr_str, connector_svg, fmt_coord,
    pass_through_attrs, resolve_fill, resolve_position, resolve_stroke, resolve_target,
    resolve_text, svg_text,
};

#[derive(Clone, Copy)]
struct LabelBox {
    center_x: f64,
    center_y: f64,
    width: f64,
    height: f64,
}

fn leader_start(
    lx: f64,
    ly: f64,
    tx: f64,
    ty: f64,
    label_box: &LabelBox,
    empty_label: bool,
) -> (f64, f64) {
    if empty_label {
        return (lx, ly);
    }

    let dx = tx - label_box.center_x;
    let dy = ty - label_box.center_y;

    let half_width = label_box.width / 2.0;
    let half_height = label_box.height / 2.0;

    if dx == 0.0 && dy == 0.0 {
        return (lx, ly);
    }

    let scale_x = if dx == 0.0 {
        f64::INFINITY
    } else {
        half_width / dx.abs()
    };
    let scale_y = if dy == 0.0 {
        f64::INFINITY
    } else {
        half_height / dy.abs()
    };
    let scale = scale_x.min(scale_y);

    let padding = 4.0;
    let length = (dx * dx + dy * dy).sqrt();

    if !scale.is_finite() || length == 0.0 {
        return (lx, ly);
    }

    (
        label_box.center_x + dx * scale + (dx / length) * padding,
        label_box.center_y + dy * scale + (dy / length) * padding,
    )
}

/// Expand `<s:callout>` into standard SVG text with optional leader line and background shape.
///
/// Supported attributes:
/// - `x`/`y` or `from` + `dx`/`dy`: label position
/// - `to`/`to-x`/`to-y`: leader line target (optional; omit for standalone label)
/// - `label`: text content
/// - `label-position`: `above`, `below`, `left`, `right`, or `auto` (default).
///   When `auto`, the label is offset away from the leader line direction so it
///   does not overlap with the line.
/// - `shape`: `none` (default), `rect`, `pill`, `circle`
/// - `curve`: leader line path type (`straight`, `elbow`, `quad`, `cubic`)
/// - `tip-style`: marker for leader line end, defaults to `s:arrow-closed`
pub fn expand(attrs: &Attrs, ctx: &mut ComponentContext) -> String {
    let pos = resolve_position(attrs, "x", "y", Some("from"), "dx", "dy", ctx.anchors);

    let Some((lx, ly)) = pos else {
        ctx.messages.push(CompilationMessage::error(
            "<s:callout> requires position (x,y or from with dx,dy)",
        ));
        return String::new();
    };

    let label = attr_str(attrs, "label", "");
    let shape = attr_str(attrs, "shape", "none");
    let pass = pass_through_attrs(attrs);
    let fill = resolve_fill(attrs, "white");
    let stroke = resolve_stroke(attrs);
    let text_fill = resolve_text(attrs);

    let target = resolve_target(attrs, ctx.anchors);

    // Background shape metrics
    let estimated_width = label.chars().count() as f64 * 7.5 + 16.0;
    let shape_height = 22.0;
    let shape_rx = match shape {
        "pill" => shape_height / 2.0,
        "circle" => estimated_width.max(shape_height) / 2.0,
        "rect" => 3.0,
        _ => 0.0,
    };

    // Determine label offset from the anchor point.
    // When a leader line exists, the label is shifted away from the line so it
    // doesn't overlap. The `label-position` attribute can override the automatic
    // direction.
    let label_position = attr_str(attrs, "label-position", "auto");
    let (text_x, text_y) = label_offset(lx, ly, target.as_ref(), label_position, shape_height);
    let label_box_width = match shape {
        "circle" => estimated_width.max(shape_height),
        _ => estimated_width,
    };
    let label_box_height = match shape {
        "circle" => label_box_width,
        _ => shape_height,
    };
    let label_box = LabelBox {
        center_x: text_x,
        center_y: text_y,
        width: label_box_width,
        height: label_box_height,
    };

    let mut svg = String::new();

    if shape != "none" {
        let _ = write!(
            svg,
            r#"<rect x="{}" y="{}" width="{}" height="{}" rx="{}" fill="{}" stroke="{}"{}/>"#,
            fmt_coord(text_x - estimated_width / 2.0),
            fmt_coord(text_y - shape_height / 2.0 - 2.0),
            fmt_coord(estimated_width),
            fmt_coord(shape_height),
            fmt_coord(shape_rx),
            fill,
            stroke,
            pass
        );
    }

    // Text label
    let text_extra = if shape == "none" {
        format!(r#" dominant-baseline="middle"{pass}"#)
    } else {
        r#" dominant-baseline="middle""#.to_string()
    };
    svg.push_str(&svg_text(
        text_x,
        text_y,
        label,
        "middle",
        12,
        text_fill,
        &text_extra,
    ));

    // Leader line (only if target is specified)
    if let Some((tx, ty)) = target {
        let tip_style = attr_str(attrs, "tip-style", "s:arrow-closed");
        let curve = attr_str(attrs, "curve", "straight");
        let marker_end = format!(r#" marker-end="url(#{tip_style})""#);
        let (line_x1, line_y1) = leader_start(lx, ly, tx, ty, &label_box, label.is_empty());
        svg.push_str(&connector_svg(
            line_x1,
            line_y1,
            tx,
            ty,
            &ConnectorOpts {
                curve,
                corner: "horizontal-first",
                marker_start: "",
                marker_end: &marker_end,
                stroke,
                pass: "",
            },
        ));
    }

    svg
}

/// Compute the label text position offset from the anchor point `(lx, ly)`.
///
/// When `position` is `"auto"` and a leader line target exists, the label is
/// shifted away from the direction the line travels so it doesn't overlap the
/// line. When there is no leader line, no offset is applied.
fn label_offset(
    lx: f64,
    ly: f64,
    target: Option<&(f64, f64)>,
    position: &str,
    shape_height: f64,
) -> (f64, f64) {
    let vertical_gap = shape_height * 0.75;

    match position {
        "above" => (lx, ly - vertical_gap),
        "below" => (lx, ly + vertical_gap),
        "left" => (lx - vertical_gap, ly),
        "right" => (lx + vertical_gap, ly),
        // "auto" or unrecognized — smart placement based on leader line direction
        _ => {
            let Some(&(tx, ty)) = target else {
                return (lx, ly);
            };

            let dx = tx - lx;
            let dy = ty - ly;

            // Determine whether the line is more vertical or horizontal
            if dy.abs() >= dx.abs() {
                // Predominantly vertical line — offset the label opposite to
                // the vertical direction of the line
                if dy >= 0.0 {
                    // Line goes down → label above
                    (lx, ly - vertical_gap)
                } else {
                    // Line goes up → label below
                    (lx, ly + vertical_gap)
                }
            } else {
                // Predominantly horizontal line — offset vertically (above)
                // since horizontal offset would look unnatural
                let _ = dx;
                (lx, ly - vertical_gap)
            }
        }
    }
}
