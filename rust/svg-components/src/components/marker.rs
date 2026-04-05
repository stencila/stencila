use super::{
    Attrs, CompilationMessage, ComponentContext, attr_f64_or, attr_str, fmt_coord,
    pass_through_attrs, resolve_position, svg_text,
};

/// Expand `<s:marker>` into a defs-backed symbol stamp with optional label.
///
/// Supported attributes:
/// - `x`/`y` or `at`: position
/// - `symbol`: symbol name (references `#s:marker-{symbol}`), defaults to `circle`
/// - `size`: symbol size in viewBox units, defaults to 20
/// - `color`: shorthand that sets both `fill` and `stroke` (overridden by explicit `fill`/`stroke`)
/// - `label`: optional text label
/// - `label-position`: `right` (default), `above`, `below`, `left`
pub fn expand(attrs: &Attrs, ctx: &mut ComponentContext) -> String {
    let Some((x, y)) = resolve_position(attrs, "x", "y", Some("at"), "dx", "dy", ctx.anchors)
    else {
        ctx.messages.push(CompilationMessage::error(
            "<s:marker> requires position (x,y or at)",
        ));
        return String::new();
    };

    let symbol = attr_str(attrs, "symbol", "circle");
    let size = attr_f64_or(attrs, "size", 20.0);
    let label = attr_str(attrs, "label", "");
    let label_position = attr_str(attrs, "label-position", "right");

    // Filter fill/stroke from pass-through since they're handled explicitly below
    let mut filtered = attrs.clone();
    filtered.remove("fill");
    filtered.remove("stroke");
    let pass = pass_through_attrs(&filtered);

    // Resolve fill and stroke: explicit fill/stroke win, then color shorthand, then currentColor
    let color = attrs.get("color").map(|s| s.as_str());
    let fill = attrs
        .get("fill")
        .map(|s| s.as_str())
        .or(color)
        .unwrap_or("currentColor");
    let stroke = attrs
        .get("stroke")
        .map(|s| s.as_str())
        .or(color)
        .unwrap_or("currentColor");
    let color_attrs = format!(" fill=\"{}\" stroke=\"{}\"", fill, stroke);

    let symbol_id = format!("s:marker-{symbol}");

    let mut svg = format!(
        "<use href=\"#{}\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\"{}{}/>",
        symbol_id,
        fmt_coord(x - size / 2.0),
        fmt_coord(y - size / 2.0),
        fmt_coord(size),
        fmt_coord(size),
        color_attrs,
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
