use std::fmt::Write;

use super::{
    Attrs, CompilationMessage, ComponentContext, attr_f64, attr_str, fmt_coord, normal_for_side,
    pass_through_attrs, resolve_position, resolve_stroke, resolve_target, resolve_text, svg_text,
    vector_metrics,
};

/// Expand `<s:brace>` into a curly brace SVG path.
///
/// The brace shape is a classic typographic curly brace with straight arms,
/// rounded corners at the endpoints, and curved transitions to a pointed tip.
/// The shape is defined in normalized coordinates and mapped to any
/// orientation via the start/end points.
///
/// Supported attributes:
/// - `from`/`to` or direct coordinates: brace endpoints
/// - `dx`/`dy`: offset from anchor
/// - `side`: `above` (default), `below`, `left`, `right` — direction the tip points
/// - `bulge`: depth of the brace perpendicular to the endpoint line (default: 10% of length)
/// - `label`: optional text label at the apex (rotated -90° for left/right)
pub fn expand(attrs: &Attrs, ctx: &mut ComponentContext) -> String {
    let start = resolve_position(attrs, "x", "y", Some("from"), "dx", "dy", ctx.anchors);
    let end = resolve_target(attrs, ctx.anchors);

    let (Some((x1, y1)), Some((x2, y2))) = (start, end) else {
        ctx.messages.push(CompilationMessage::error(
            "<s:brace> requires start and end coordinates (from/to or x,y + to-x,to-y)",
        ));
        return String::new();
    };

    let side = attr_str(attrs, "side", "above");
    let label = attrs.get("label");
    let pass = pass_through_attrs(attrs);
    let stroke = resolve_stroke(attrs);
    let text_fill = resolve_text(attrs);

    // Direction vector from start to end
    let metrics = vector_metrics(x1, y1, x2, y2);

    if metrics.len < 0.001 {
        ctx.messages.push(CompilationMessage::warning(
            "<s:brace> start and end points are too close together",
        ));
        return String::new();
    }

    // Bulge amount: user-specified or 10% of the span length
    let bulge = attr_f64(attrs, "bulge").unwrap_or(metrics.len * 0.1);

    // Normal direction: the side name indicates which direction the tip points.
    // For a horizontal line (left→right): above = up, below = down
    // For a vertical line (top→bottom): left = left, right = right
    let (nx, ny) = normal_for_side(&metrics, side);

    // Map normalized (t, n) coordinates to actual SVG coordinates.
    // t: fraction along the spine (0 = start, 1 = end)
    // n: fraction of bulge depth (0 = on chord, 1 = at tip)
    let pt = |t: f64, n: f64| {
        let x = (x1 + t * metrics.dx + n * bulge * nx).round();
        let y = (y1 + t * metrics.dy + n * bulge * ny).round();
        format!("{},{}", fmt_coord(x), fmt_coord(y))
    };

    // The brace shape in normalized (t, n) coordinates, derived from a
    // reference typographic curly brace path. The shape has:
    //   - Rounded corners at the endpoints
    //   - Straight arms at n=0.4 (40% of bulge depth)
    //   - Curved transitions from the arms to the pointed tip at n=0.82
    let r = 0.048; // corner radius as fraction of spine length
    let mut d = String::new();
    let segments: &[(&str, &[(f64, f64)])] = &[
        ("M", &[(0.0, 0.0)]),
        ("C", &[(0.0, 0.0), (0.0, 0.4), (r, 0.4)]),
        ("L", &[(0.417, 0.4)]),
        ("C", &[(0.451, 0.4), (0.466, 0.4), (0.5, 0.82)]),
        ("C", &[(0.534, 0.4), (0.549, 0.4), (0.583, 0.4)]),
        ("L", &[(1.0 - r, 0.4)]),
        ("C", &[(1.0, 0.4), (1.0, 0.0), (1.0, 0.0)]),
    ];
    for (cmd, points) in segments {
        write!(
            d,
            " {} {}",
            cmd,
            points
                .iter()
                .map(|&(t, n)| pt(t, n))
                .collect::<Vec<_>>()
                .join(" ")
        )
        .ok();
    }

    let path = format!(r#"<path d="{d}" fill="none" stroke="{stroke}"{pass}/>"#,);

    let label_svg = match label {
        Some(label_text) => {
            let label_offset = 12.0;
            let lx = x1 + 0.5 * metrics.dx + (0.82 * bulge + label_offset) * nx;
            let ly = y1 + 0.5 * metrics.dy + (0.82 * bulge + label_offset) * ny;
            let extra = match side {
                "left" | "right" => format!(
                    r#" dominant-baseline="middle" transform="rotate(-90,{},{})""#,
                    fmt_coord(lx),
                    fmt_coord(ly)
                ),
                _ => r#" dominant-baseline="middle""#.to_string(),
            };
            svg_text(lx, ly, label_text, "middle", 12, text_fill, &extra)
        }
        None => String::new(),
    };

    format!("{path}{label_svg}")
}
