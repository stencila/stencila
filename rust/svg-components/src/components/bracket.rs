use std::fmt::Write;

use super::{
    Attrs, CompilationMessage, ComponentContext, attr_f64, attr_str, fmt_coord, normal_for_side,
    pass_through_attrs, resolve_position, resolve_stroke, resolve_target, resolve_text, svg_text,
    vector_metrics,
};

/// Expand `<s:bracket>` into a square or round bracket SVG path.
///
/// Renders a bracket (square or round) between two points with an optional
/// label at the midpoint. Square brackets have right-angle corners; round
/// brackets use quadratic curves.
///
/// Supported attributes:
/// - `from`/`to` or direct coordinates: bracket endpoints
/// - `dx`/`dy`: offset from anchor
/// - `side`: `above` (default), `below`, `left`, `right` — direction the bracket opens toward
/// - `depth`: depth of the bracket perpendicular to the endpoint line (default: 8% of length)
/// - `variant`: `square` (default) or `round`
/// - `label`: optional text label at the midpoint
pub fn expand(attrs: &Attrs, ctx: &mut ComponentContext) -> String {
    let start = resolve_position(attrs, "x", "y", Some("from"), "dx", "dy", ctx.anchors);
    let end = resolve_target(attrs, ctx.anchors);

    let (Some((x1, y1)), Some((x2, y2))) = (start, end) else {
        ctx.messages.push(CompilationMessage::error(
            "<s:bracket> requires start and end coordinates (from/to or x,y + to-x,to-y)",
        ));
        return String::new();
    };

    let side = attr_str(attrs, "side", "above");
    let variant = attr_str(attrs, "variant", "square");
    let label = attrs.get("label");
    let pass = pass_through_attrs(attrs);
    let stroke = resolve_stroke(attrs);
    let text_fill = resolve_text(attrs);

    let metrics = vector_metrics(x1, y1, x2, y2);

    if metrics.len < 0.001 {
        ctx.messages.push(CompilationMessage::warning(
            "<s:bracket> start and end points are too close together",
        ));
        return String::new();
    }

    let depth = attr_f64(attrs, "depth").unwrap_or(metrics.len * 0.08);

    // Normal direction: the side name indicates which side of the content the
    // bracket sits on. The spine (connecting line) is offset in this direction,
    // while the arms point back toward the chord (toward the content).
    // For a horizontal left-to-right line: above = spine is up, arms point down.
    let (nx, ny) = normal_for_side(&metrics, side);

    // Spine is offset in the normal direction (the side the bracket sits on).
    // Arms point back from the spine toward the chord (toward the content).
    let sx1 = x1 + depth * nx;
    let sy1 = y1 + depth * ny;
    let sx2 = x2 + depth * nx;
    let sy2 = y2 + depth * ny;

    let path = if variant == "round" {
        // Round bracket: quadratic curves at the corners
        let r = depth.min(metrics.len * 0.15); // corner radius
        // Tangent unit vector along the spine
        let tx = metrics.dx / metrics.len;
        let ty = metrics.dy / metrics.len;

        let mut d = String::new();
        // Start at the chord end of the first arm
        write!(d, "M {},{}", fmt_coord(x1), fmt_coord(y1)).ok();
        // Curve into the spine at start
        write!(
            d,
            " Q {},{} {},{}",
            fmt_coord(sx1),
            fmt_coord(sy1),
            fmt_coord(sx1 + r * tx),
            fmt_coord(sy1 + r * ty)
        )
        .ok();
        // Line along spine to near the end
        write!(
            d,
            " L {},{}",
            fmt_coord(sx2 - r * tx),
            fmt_coord(sy2 - r * ty)
        )
        .ok();
        // Curve out at end
        write!(
            d,
            " Q {},{} {},{}",
            fmt_coord(sx2),
            fmt_coord(sy2),
            fmt_coord(x2),
            fmt_coord(y2)
        )
        .ok();

        format!(r#"<path d="{d}" fill="none" stroke="{stroke}"{pass}/>"#)
    } else {
        // Square bracket: right-angle corners
        let mut d = String::new();
        write!(d, "M {},{}", fmt_coord(x1), fmt_coord(y1)).ok();
        write!(d, " L {},{}", fmt_coord(sx1), fmt_coord(sy1)).ok();
        write!(d, " L {},{}", fmt_coord(sx2), fmt_coord(sy2)).ok();
        write!(d, " L {},{}", fmt_coord(x2), fmt_coord(y2)).ok();

        format!(r#"<path d="{d}" fill="none" stroke="{stroke}"{pass}/>"#)
    };

    let label_svg = match label {
        Some(label_text) => {
            let label_offset = 12.0;
            // Label at midpoint of spine, offset in the bracket direction
            let mx = f64::midpoint(x1, x2);
            let my = f64::midpoint(y1, y2);
            let lx = mx + (depth + label_offset) * nx;
            let ly = my + (depth + label_offset) * ny;
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
