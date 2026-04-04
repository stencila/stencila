use super::{
    Attrs, CompilationMessage, ComponentContext, attr_f64, attr_f64_or, attr_str, fmt_coord,
    pass_through_attrs,
};

/// Expand `<s:halo>` into a glowing ring around a point.
///
/// Renders a semi-transparent ring (or series of concentric rings) to
/// create a glow/highlight effect around a specific location.
///
/// Supported attributes:
/// - `cx`/`cy`: center position
/// - `r`: inner radius of the halo ring (default: 15)
/// - `width`: ring width (default: 8)
/// - `color`: ring color (default: currentColor)
/// - `opacity`: ring opacity (default: 0.4)
pub fn expand(attrs: &Attrs, ctx: &mut ComponentContext) -> String {
    let (Some(cx), Some(cy)) = (attr_f64(attrs, "cx"), attr_f64(attrs, "cy")) else {
        ctx.messages.push(CompilationMessage::error(
            "<s:halo> requires 'cx' and 'cy' attributes",
        ));
        return String::new();
    };

    let r = attr_f64_or(attrs, "r", 15.0);
    let width = attr_f64_or(attrs, "width", 8.0);
    let color = attr_str(attrs, "color", "currentColor");
    let opacity = attr_f64_or(attrs, "opacity", 0.4);
    let pass = pass_through_attrs(attrs);

    // Draw the halo as a circle with thick stroke at the specified radius.
    // The circle radius is at the center of the ring.
    let ring_r = r + width / 2.0;

    format!(
        r#"<circle cx="{}" cy="{}" r="{}" fill="none" stroke="{}" stroke-width="{}" opacity="{}"{pass}/>"#,
        fmt_coord(cx),
        fmt_coord(cy),
        fmt_coord(ring_r),
        crate::compile::xml_escape(color),
        fmt_coord(width),
        opacity,
    )
}
