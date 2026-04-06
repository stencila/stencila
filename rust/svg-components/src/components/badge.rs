use std::fmt::Write;

use super::{
    Attrs, CompilationMessage, ComponentContext, attr_str, fmt_coord, pass_through_attrs,
    resolve_fill, resolve_position, resolve_stroke, resolve_text, svg_text,
};

/// Expand `<s:badge>` into standard SVG text with a pill-shaped background.
///
/// Badge is a convenience component wrapping `<s:callout>` with these defaults:
/// - `shape="pill"` (always)
/// - No leader line (to/to-x/to-y ignored)
/// - Smaller font size
///
/// Supported attributes:
/// - `x`/`y` or `from` + `dx`/`dy`: position
/// - `label`: text content
pub fn expand(attrs: &Attrs, ctx: &mut ComponentContext) -> String {
    let pos = resolve_position(attrs, "x", "y", Some("from"), "dx", "dy", ctx.anchors);

    let Some((lx, ly)) = pos else {
        ctx.messages.push(CompilationMessage::error(
            "<s:badge> requires position (x,y or from with dx,dy)",
        ));
        return String::new();
    };

    let label = attr_str(attrs, "label", "");
    let pass = pass_through_attrs(attrs);
    let fill = resolve_fill(attrs, "white");
    let stroke = resolve_stroke(attrs);
    let text_fill = resolve_text(attrs);

    let estimated_width = label.chars().count() as f64 * 6.5 + 12.0;
    let shape_height = 18.0;
    let shape_rx = shape_height / 2.0;

    let mut svg = String::new();

    // Pill background
    let _ = write!(
        svg,
        r#"<rect x="{}" y="{}" width="{}" height="{}" rx="{}" fill="{}" stroke="{}"{}/>"#,
        fmt_coord(lx - estimated_width / 2.0),
        fmt_coord(ly - shape_height / 2.0),
        fmt_coord(estimated_width),
        fmt_coord(shape_height),
        fmt_coord(shape_rx),
        fill,
        stroke,
        pass
    );

    // Text label
    svg.push_str(&svg_text(
        lx,
        ly,
        label,
        "middle",
        10,
        text_fill,
        r#" dominant-baseline="middle""#,
    ));

    svg
}
