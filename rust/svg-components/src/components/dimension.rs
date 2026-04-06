use super::{
    Attrs, CompilationMessage, ComponentContext, attr_str, fmt_coord, pass_through_attrs,
    resolve_position, resolve_stroke, resolve_target, svg_line, svg_text, vector_metrics,
};

/// Expand `<s:dimension>` into standard SVG.
///
/// Renders a dimension line between two points with end caps and a label.
///
/// Supported attributes:
/// - `from`/`to` or `x`/`y` + `to-x`/`to-y`: start and end points
/// - `dx`/`dy`: offset from anchor
/// - `label`: text content (e.g. "4.2 cm")
/// - `label-position`: `above` (default) or `below`
/// - `label-angle`: `along` (default), `horizontal`, `vertical`, or a number in degrees
/// - `side`: `above` (default), `below` — offset direction for the dimension line
#[allow(clippy::too_many_lines)]
pub fn expand(attrs: &Attrs, ctx: &mut ComponentContext) -> String {
    let start = resolve_position(attrs, "x", "y", Some("from"), "dx", "dy", ctx.anchors);
    let end = resolve_target(attrs, ctx.anchors);

    let (Some((x1, y1)), Some((x2, y2))) = (start, end) else {
        ctx.messages.push(CompilationMessage::error(
            "<s:dimension> requires start and end coordinates",
        ));
        return String::new();
    };

    let label = attr_str(attrs, "label", "");
    let label_position = attr_str(attrs, "label-position", "above");
    let label_angle = attr_str(attrs, "label-angle", "along");
    let side = attr_str(attrs, "side", "above");
    let pass = pass_through_attrs(attrs);
    let stroke = resolve_stroke(attrs);
    let cap_height = 8.0;
    let side_offset = 20.0;

    let metrics = vector_metrics(x1, y1, x2, y2);
    let (nx, ny) = (metrics.nx, metrics.ny);

    let side_sign = match side {
        "below" => -1.0,
        _ => 1.0,
    };

    // Offset dimension line endpoints
    let ox = nx * side_offset * side_sign;
    let oy = ny * side_offset * side_sign;
    let dx1 = x1 + ox;
    let dy1 = y1 + oy;
    let dx2 = x2 + ox;
    let dy2 = y2 + oy;

    // Main dimension line (offset)
    let mut svg = svg_line(dx1, dy1, dx2, dy2, stroke, &pass);

    // Extension lines from original points to offset dimension line endpoints
    if metrics.len > 0.0 {
        svg.push_str(&svg_line(x1, y1, dx1, dy1, stroke, ""));
        svg.push_str(&svg_line(x2, y2, dx2, dy2, stroke, ""));

        // Cap lines (short perpendicular lines at each end of the offset dimension line)
        let cx = nx * cap_height / 2.0;
        let cy = ny * cap_height / 2.0;
        svg.push_str(&svg_line(
            dx1 - cx,
            dy1 - cy,
            dx1 + cx,
            dy1 + cy,
            stroke,
            "",
        ));
        svg.push_str(&svg_line(
            dx2 - cx,
            dy2 - cy,
            dx2 + cx,
            dy2 + cy,
            stroke,
            "",
        ));
    }

    // Label at midpoint of offset dimension line
    if !label.is_empty() {
        let mx = f64::midpoint(dx1, dx2);
        let my = f64::midpoint(dy1, dy2);

        // Tangent angle of the dimension line
        let tangent_deg = if metrics.len > 0.0 {
            metrics.dy.atan2(metrics.dx).to_degrees()
        } else {
            0.0
        };

        // Resolve label rotation angle
        let angle_deg = match label_angle {
            "horizontal" => 0.0,
            "vertical" => 90.0,
            "along" => {
                let mut deg = tangent_deg;
                // Keep text right-side-up: flip if pointing leftward
                if deg > 90.0 {
                    deg -= 180.0;
                } else if deg < -90.0 {
                    deg += 180.0;
                }
                deg
            }
            other => other.parse::<f64>().unwrap_or(0.0),
        };

        // Perpendicular offset relative to the label's reading direction
        let perp_dist = 10.0;
        let read_rad = angle_deg.to_radians();
        let lnx = read_rad.sin();
        let lny = -read_rad.cos();

        let label_sign = if label_position == "below" { -1.0 } else { 1.0 };
        let lx = mx + lnx * perp_dist * label_sign;
        let ly = my + lny * perp_dist * label_sign;

        let baseline = if label_position == "below" {
            "hanging"
        } else {
            "auto"
        };

        if angle_deg.abs() < 0.1 {
            svg.push_str(&svg_text(
                lx,
                ly,
                label,
                "middle",
                12,
                &format!(r#" dominant-baseline="{baseline}""#),
            ));
        } else {
            svg.push_str(&svg_text(
                lx,
                ly,
                label,
                "middle",
                12,
                &format!(
                    r#" dominant-baseline="{baseline}" transform="rotate({}, {}, {})""#,
                    fmt_coord(angle_deg),
                    fmt_coord(lx),
                    fmt_coord(ly)
                ),
            ));
        }
    }

    svg
}
