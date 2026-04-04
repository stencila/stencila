use super::*;
use std::f64::consts::{PI, TAU};

/// Expand `<s:angle>` into an arc showing the angle between two lines.
///
/// Renders an arc at a vertex point showing the angle formed by two lines
/// extending from the vertex to `from` and `to` points. An optional label
/// is placed along the arc.
///
/// Supported attributes:
/// - `x`/`y` or `at`: the vertex position
/// - `from`: anchor or coordinates for the first ray endpoint
/// - `to`: anchor or coordinates for the second ray endpoint
/// - `r`: arc radius in viewBox units (default: 30)
/// - `label`: optional text label (e.g., "45°", "θ")
pub fn expand(attrs: &Attrs, ctx: &mut ComponentContext) -> String {
    // Vertex position
    let vertex = resolve_position(attrs, "x", "y", Some("at"), "dx", "dy", ctx.anchors);
    let Some((vx, vy)) = vertex else {
        ctx.messages.push(CompilationMessage::error(
            "<s:angle> requires vertex position (x,y or at)",
        ));
        return String::new();
    };

    // from point (first ray endpoint)
    let from = if let Some(anchor_ref) = attrs.get("from") {
        crate::anchors::resolve_anchor_ref(anchor_ref, ctx.anchors, 0.0, 0.0)
    } else if let (Some(fx), Some(fy)) = (attr_f64(attrs, "from-x"), attr_f64(attrs, "from-y")) {
        Some((fx, fy))
    } else {
        None
    };

    // to point (second ray endpoint)
    let to = resolve_target(attrs, ctx.anchors);

    let (Some((fx, fy)), Some((tx, ty))) = (from, to) else {
        ctx.messages.push(CompilationMessage::error(
            "<s:angle> requires 'from' and 'to' points for the two rays",
        ));
        return String::new();
    };

    let r = attr_f64_or(attrs, "r", 30.0);
    let label = attrs.get("label");
    let pass = pass_through_attrs(attrs);

    // Compute angles from vertex to each endpoint
    let angle_from = (fy - vy).atan2(fx - vx);
    let angle_to = (ty - vy).atan2(tx - vx);

    // Normalize: sweep from angle_from to angle_to going counterclockwise
    // Use the shorter arc (< 180°)
    let mut sweep_angle = angle_to - angle_from;
    // Normalize to [-PI, PI]
    sweep_angle = (sweep_angle + PI).rem_euclid(TAU) - PI;

    // Arc start and end points on the circle of radius r centered at vertex
    let arc_x1 = vx + r * angle_from.cos();
    let arc_y1 = vy + r * angle_from.sin();
    let arc_x2 = vx + r * angle_to.cos();
    let arc_y2 = vy + r * angle_to.sin();

    // SVG arc flags
    let large_arc = if sweep_angle.abs() > PI { 1 } else { 0 };
    let sweep_flag = if sweep_angle > 0.0 { 1 } else { 0 };

    let path = format!(
        r#"<path d="M {},{} A {},{} 0 {} {} {},{}" fill="none" stroke="currentColor"{pass}/>"#,
        fmt_coord(arc_x1),
        fmt_coord(arc_y1),
        fmt_coord(r),
        fmt_coord(r),
        large_arc,
        sweep_flag,
        fmt_coord(arc_x2),
        fmt_coord(arc_y2),
    );

    let label_svg = match label {
        Some(label_text) => {
            // Place label at the midpoint of the arc
            let mid_angle = angle_from + sweep_angle / 2.0;
            let label_r = r + 12.0;
            let lx = vx + label_r * mid_angle.cos();
            let ly = vy + label_r * mid_angle.sin();
            svg_text(
                lx,
                ly,
                label_text,
                "middle",
                12,
                r#" dominant-baseline="middle""#,
            )
        }
        None => String::new(),
    };

    format!("{path}{label_svg}")
}
