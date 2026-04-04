use super::*;

/// Expand `<s:crosshair>` into a crosshair/reticle SVG.
///
/// Renders perpendicular lines centered on a point, with an optional gap
/// in the center and an optional enclosing ring.
///
/// Supported attributes:
/// - `cx`/`cy`: center position
/// - `size`: overall size (arm length from center), default 20
/// - `gap`: gap radius around center where lines are not drawn, default 4
/// - `ring`: `true` to draw an enclosing circle at the arm endpoints
/// - `label`: optional text label
/// - `label-position`: `right` (default), `above`, `below`, `left`
pub fn expand(attrs: &Attrs, ctx: &mut ComponentContext) -> String {
    let (Some(cx), Some(cy)) = (attr_f64(attrs, "cx"), attr_f64(attrs, "cy")) else {
        ctx.messages.push(CompilationMessage::error(
            "<s:crosshair> requires 'cx' and 'cy' attributes",
        ));
        return String::new();
    };

    let size = attr_f64_or(attrs, "size", 20.0);
    let gap = attr_f64_or(attrs, "gap", 4.0);
    let ring = attr_str(attrs, "ring", "false") == "true";
    let label = attr_str(attrs, "label", "");
    let label_position = attr_str(attrs, "label-position", "right");
    let pass = pass_through_attrs(attrs);

    let mut svg = String::new();

    // Horizontal crosshair lines (with gap)
    svg.push_str(&svg_line(cx - size, cy, cx - gap, cy, &pass));
    svg.push_str(&svg_line(cx + gap, cy, cx + size, cy, ""));

    // Vertical crosshair lines (with gap)
    svg.push_str(&svg_line(cx, cy - size, cx, cy - gap, ""));
    svg.push_str(&svg_line(cx, cy + gap, cx, cy + size, ""));

    // Optional enclosing ring
    if ring {
        svg.push_str(&format!(
            r#"<circle cx="{}" cy="{}" r="{}" fill="none" stroke="currentColor"/>"#,
            fmt_coord(cx),
            fmt_coord(cy),
            fmt_coord(size),
        ));
    }

    // Optional label
    if !label.is_empty() {
        let (lx, ly, anchor) = match label_position {
            "above" => (cx, cy - size - 6.0, "middle"),
            "below" => (cx, cy + size + 14.0, "middle"),
            "left" => (cx - size - 6.0, cy + 4.0, "end"),
            _ => (cx + size + 6.0, cy + 4.0, "start"), // right (default)
        };
        svg.push_str(&svg_text(lx, ly, label, anchor, 12, ""));
    }

    svg
}
