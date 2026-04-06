use super::{
    Attrs, CompilationMessage, ComponentContext, ConnectorOpts, CubicControlPoints,
    QuadControlPoint, attr_str, connector_svg, cubic_control_points, fmt_coord, marker_attrs,
    pass_through_attrs, quad_control_point, resolve_position, resolve_stroke, resolve_target,
    resolve_text,
    svg_text, vector_metrics,
};

/// Expand `<s:arrow>` into standard SVG path/line with marker references.
///
/// Supported attributes:
/// - `from`/`to` or `x`/`y` + `to-x`/`to-y`: start and end coordinates
/// - `dx`/`dy`: offset from anchor
/// - `curve`: `straight` (default), `elbow`, `quad`, `cubic`
/// - `corner`: `horizontal-first` (default) or `vertical-first` (for elbow)
/// - `tip`: `end` (default), `start`, `both`, `none`
/// - `tip-style`: marker id, defaults to `s:arrow-closed`
/// - `label`: optional text label placed at midpoint
/// - `label-position`: `above` (default), `below` — perpendicular offset from the line
/// - `label-angle`: `along` (default), `horizontal`, `vertical`, or a number in degrees
pub fn expand(attrs: &Attrs, ctx: &mut ComponentContext) -> String {
    let start = resolve_position(attrs, "x", "y", Some("from"), "dx", "dy", ctx.anchors);
    let end = resolve_target(attrs, ctx.anchors);

    let (Some((x1, y1)), Some((x2, y2))) = (start, end) else {
        ctx.messages.push(CompilationMessage::error(
            "<s:arrow> requires start (from/x,y) and end (to/to-x,to-y) coordinates",
        ));
        return String::new();
    };

    let curve = attr_str(attrs, "curve", "straight");
    let tip = attr_str(attrs, "tip", "end");
    let tip_style = attr_str(attrs, "tip-style", "s:arrow-closed");
    let corner = attr_str(attrs, "corner", "horizontal-first");
    let label = attrs.get("label");
    let label_position = attr_str(attrs, "label-position", "above");
    let label_angle = attr_str(attrs, "label-angle", "along");
    let pass = pass_through_attrs(attrs);
    let stroke = resolve_stroke(attrs);
    let text_fill = resolve_text(attrs);

    let (ms, me) = marker_attrs(tip, tip_style);
    let path = connector_svg(
        x1,
        y1,
        x2,
        y2,
        &ConnectorOpts {
            curve,
            corner,
            marker_start: &ms,
            marker_end: &me,
            stroke,
            pass: &pass,
        },
    );

    let label_svg = match label {
        Some(text) => {
            let (mx, my, tangent_deg) = curve_midpoint_and_tangent(x1, y1, x2, y2, curve, corner);
            arrow_label_svg(mx, my, tangent_deg, text, label_position, label_angle, text_fill)
        }
        None => String::new(),
    };

    format!("{path}{label_svg}")
}

/// Compute the position on the curve at t=0.5 and the tangent angle (in
/// degrees) at that point. For straight lines this is the chord midpoint
/// and chord angle; for Bézier curves it uses the proper parametric
/// derivatives.
fn curve_midpoint_and_tangent(
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    curve: &str,
    corner: &str,
) -> (f64, f64, f64) {
    match curve {
        "quad" => {
            // Quadratic Bézier: M P0 Q P1 P2
            let QuadControlPoint { cx, cy } = quad_control_point(x1, y1, x2, y2);
            // B(0.5)  = 0.25*P0 + 0.5*P1 + 0.25*P2
            let mx = 0.25 * x1 + 0.5 * cx + 0.25 * x2;
            let my = 0.25 * y1 + 0.5 * cy + 0.25 * y2;
            // B'(t)   = 2(1-t)(P1-P0) + 2t(P2-P1)  →  at t=0.5: P2-P0
            let tx = x2 - x1;
            let ty = y2 - y1;
            (mx, my, ty.atan2(tx).to_degrees())
        }
        "cubic" => {
            // Cubic Bézier: M P0 C P1 P2 P3
            let CubicControlPoints { cx1, cy1, cx2, cy2 } = cubic_control_points(x1, y1, x2, y2);
            // B(0.5)  = (P0 + 3P1 + 3P2 + P3) / 8
            let mx = (x1 + 3.0 * cx1 + 3.0 * cx2 + x2) / 8.0;
            let my = (y1 + 3.0 * cy1 + 3.0 * cy2 + y2) / 8.0;
            // B'(t)   = 3(1-t)²(P1-P0) + 6(1-t)t(P2-P1) + 3t²(P3-P2)
            // At t=0.5: 0.75*(P1-P0) + 1.5*(P2-P1) + 0.75*(P3-P2)
            //         simplified: (-P0 - P1 + P2 + P3) * 0.75
            let tx = -x1 - cx1 + cx2 + x2;
            let ty = -y1 - cy1 + cy2 + y2;
            (mx, my, ty.atan2(tx).to_degrees())
        }
        "elbow" => {
            // Polyline: P0 → bend → P2
            // Place label at the midpoint of the first segment (always
            // horizontal or vertical) with a tangent angle of 0°. This
            // keeps the label horizontal and readable, avoiding vertical
            // text when the polyline midpoint falls on a vertical segment.
            let (bend_x, bend_y) = if corner == "vertical-first" {
                (x1, y2)
            } else {
                (x2, y1)
            };
            let mx = f64::midpoint(x1, bend_x);
            let my = f64::midpoint(y1, bend_y);
            // Use the first segment's direction for the tangent
            let tx = bend_x - x1;
            let ty = bend_y - y1;
            let deg = if tx.abs() > f64::EPSILON || ty.abs() > f64::EPSILON {
                ty.atan2(tx).to_degrees()
            } else {
                0.0
            };
            (mx, my, deg)
        }
        _ => {
            // Straight line
            let metrics = vector_metrics(x1, y1, x2, y2);
            let deg = if metrics.len > 0.0 {
                metrics.dy.atan2(metrics.dx).to_degrees()
            } else {
                0.0
            };
            (f64::midpoint(x1, x2), f64::midpoint(y1, y2), deg)
        }
    }
}

/// Render an arrow label at a given position with a given tangent angle,
/// offset perpendicular to the reading direction.
fn arrow_label_svg(
    mx: f64,
    my: f64,
    tangent_deg: f64,
    text: &str,
    label_position: &str,
    label_angle: &str,
    text_fill: &str,
) -> String {
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

    // Perpendicular offset relative to the label's reading direction so that
    // "above" always means visually above the text regardless of arrow direction.
    // Rotate the reading direction 90° counter-clockwise in SVG coordinates
    // (where y points down) to get the "above" normal.
    let perp_dist = 10.0;
    let read_rad = angle_deg.to_radians();
    // 90° CCW in SVG coords: (sin θ, -cos θ) points "above" the reading direction
    let nx = read_rad.sin();
    let ny = -read_rad.cos();

    let side_sign = if label_position == "below" { -1.0 } else { 1.0 };
    let lx = mx + nx * perp_dist * side_sign;
    let ly = my + ny * perp_dist * side_sign;

    // Use a dominant-baseline so the text sits above or below the offset point
    let baseline = if label_position == "below" {
        "hanging"
    } else {
        "auto"
    };

    if angle_deg.abs() < 0.1 {
        svg_text(
            lx,
            ly,
            text,
            "middle",
            12,
            text_fill,
            &format!(r#" dominant-baseline="{baseline}""#),
        )
    } else {
        svg_text(
            lx,
            ly,
            text,
            "middle",
            12,
            text_fill,
            &format!(
                r#" dominant-baseline="{baseline}" transform="rotate({}, {}, {})""#,
                fmt_coord(angle_deg),
                fmt_coord(lx),
                fmt_coord(ly)
            ),
        )
    }
}
