use super::*;

/// Expand `<s:marker>` into a defs-backed symbol stamp with optional label.
///
/// Supported attributes:
/// - `x`/`y`: position
/// - `symbol`: symbol name (references `#s:marker-{symbol}`), defaults to `circle`
/// - `size`: symbol size in viewBox units, defaults to 20
/// - `label`: optional text label
/// - `label-position`: `right` (default), `above`, `below`, `left`
pub fn expand(attrs: &Attrs, ctx: &mut ComponentContext) -> String {
    let (Some(x), Some(y)) = (attr_f64(attrs, "x"), attr_f64(attrs, "y")) else {
        ctx.messages.push(CompilationMessage::error(
            "<s:marker> requires 'x' and 'y' attributes",
        ));
        return String::new();
    };

    let symbol = attr_str(attrs, "symbol", "circle");
    let size = attr_f64_or(attrs, "size", 20.0);
    let label = attr_str(attrs, "label", "");
    let label_position = attr_str(attrs, "label-position", "right");
    let pass = pass_through_attrs(attrs);

    let symbol_id = format!("s:marker-{symbol}");

    let mut svg = format!(
        "<use href=\"#{}\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\"{}/>",
        symbol_id,
        fmt_coord(x - size / 2.0),
        fmt_coord(y - size / 2.0),
        fmt_coord(size),
        fmt_coord(size),
        pass
    );

    if !label.is_empty() {
        let (lx, ly, anchor) = match label_position {
            "above" => (x, y - size / 2.0 - 4.0, "middle"),
            "below" => (x, y + size / 2.0 + 14.0, "middle"),
            "left" => (x - size / 2.0 - 4.0, y + 4.0, "end"),
            _ => (x + size / 2.0 + 4.0, y + 4.0, "start"), // right (default)
        };
        svg.push_str(&svg_text(lx, ly, label, anchor, 12, ""));
    }

    svg
}
