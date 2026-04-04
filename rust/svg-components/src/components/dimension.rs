use super::*;

/// Expand `<s:dimension>` into standard SVG.
///
/// Renders a dimension line between two points with end caps and a label.
///
/// Supported attributes:
/// - `from`/`to` or `x`/`y` + `to-x`/`to-y`: start and end points
/// - `dx`/`dy`: offset from anchor
/// - `label`: text content (e.g. "4.2 cm")
/// - `label-position`: `above` (default) or `below`
/// - `side`: `above` (default), `below` — offset direction for the dimension line
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
    let side = attr_str(attrs, "side", "above");
    let pass = pass_through_attrs(attrs);
    let cap_height = 8.0;
    let side_offset = 20.0;

    let dx = x2 - x1;
    let dy = y2 - y1;
    let len = (dx * dx + dy * dy).sqrt();

    // Perpendicular unit normal pointing "above" (left of start→end direction).
    // In SVG coordinates (-dy, dx)/len points upward for a left-to-right line.
    let (nx, ny) = if len > 0.0 {
        (-dy / len, dx / len)
    } else {
        (0.0, -1.0)
    };

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
    let mut svg = svg_line(
        dx1,
        dy1,
        dx2,
        dy2,
        &format!(r#" marker-start="url(#s:cap-line)" marker-end="url(#s:cap-line)"{pass}"#),
    );

    // Extension lines from original points to offset dimension line endpoints
    if len > 0.0 {
        svg.push_str(&svg_line(x1, y1, dx1, dy1, ""));
        svg.push_str(&svg_line(x2, y2, dx2, dy2, ""));

        // Cap lines (short perpendicular lines at each end of the offset dimension line)
        let cx = nx * cap_height / 2.0;
        let cy = ny * cap_height / 2.0;
        svg.push_str(&svg_line(dx1 - cx, dy1 - cy, dx1 + cx, dy1 + cy, ""));
        svg.push_str(&svg_line(dx2 - cx, dy2 - cy, dx2 + cx, dy2 + cy, ""));
    }

    // Label at midpoint of offset dimension line
    if !label.is_empty() {
        let mx = (dx1 + dx2) / 2.0;
        let my = (dy1 + dy2) / 2.0;
        let label_offset = match label_position {
            "below" => 16.0,
            _ => -8.0,
        };
        svg.push_str(&svg_text(mx, my + label_offset, label, "middle", 12, ""));
    }

    svg
}
