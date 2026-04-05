use super::{
    Attrs, CompilationMessage, ComponentContext, attr_f64, attr_str, pass_through_attrs,
    resolve_position, svg_line, svg_text,
};

/// Expand `<s:scale-bar>` into standard SVG.
///
/// Renders a horizontal or vertical bar with end caps and a centered label.
///
/// Supported attributes:
/// - `x`/`y` or `at`: position (left/top of bar)
/// - `length`: bar length in viewBox units
/// - `label`: text content (e.g. "20 μm")
/// - `label-position`: `below` (default) or `above`
/// - `side`: `bottom` (default), `top`, `left`, `right` — controls orientation
pub fn expand(attrs: &Attrs, ctx: &mut ComponentContext) -> String {
    let (Some((x, y)), Some(length)) = (
        resolve_position(attrs, "x", "y", Some("at"), "dx", "dy", ctx.anchors),
        attr_f64(attrs, "length"),
    ) else {
        ctx.messages.push(CompilationMessage::error(
            "<s:scale-bar> requires position (x,y or at) and 'length' attribute",
        ));
        return String::new();
    };

    let label = attr_str(attrs, "label", "");
    let label_position = attr_str(attrs, "label-position", "below");
    let side = attr_str(attrs, "side", "bottom");
    let pass = pass_through_attrs(attrs);
    let cap_height = 8.0;

    let vertical = matches!(side, "left" | "right");

    let mut svg = if vertical {
        // Main vertical bar
        svg_line(x, y, x, y + length, &pass)
    } else {
        // Main horizontal bar
        svg_line(x, y, x + length, y, &pass)
    };

    if vertical {
        // Top end cap (horizontal)
        svg.push_str(&svg_line(
            x - cap_height / 2.0,
            y,
            x + cap_height / 2.0,
            y,
            "",
        ));

        // Bottom end cap (horizontal)
        svg.push_str(&svg_line(
            x - cap_height / 2.0,
            y + length,
            x + cap_height / 2.0,
            y + length,
            "",
        ));
    } else {
        // Left end cap (vertical)
        svg.push_str(&svg_line(
            x,
            y - cap_height / 2.0,
            x,
            y + cap_height / 2.0,
            "",
        ));

        // Right end cap (vertical)
        svg.push_str(&svg_line(
            x + length,
            y - cap_height / 2.0,
            x + length,
            y + cap_height / 2.0,
            "",
        ));
    }

    // Label
    if !label.is_empty() {
        if vertical {
            let text_y = y + length / 2.0;
            let text_x = match side {
                "left" => x - 8.0,
                _ => x + 8.0, // right
            };
            let anchor = match side {
                "left" => "end",
                _ => "start", // right
            };
            svg.push_str(&svg_text(text_x, text_y, label, anchor, 12, ""));
        } else {
            let text_x = x + length / 2.0;
            let text_y = match label_position {
                "above" => y - 8.0,
                _ => y + 16.0, // below (default)
            };
            svg.push_str(&svg_text(text_x, text_y, label, "middle", 12, ""));
        }
    }

    svg
}
