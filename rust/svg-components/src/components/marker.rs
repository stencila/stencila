use std::fmt::Write;

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
/// - `background`: background color behind symbol and label (default `white`, `none` to disable)
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
    let background = attr_str(attrs, "background", "white");

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

    // Label position and text anchor
    let (lx, ly, anchor) = if label.is_empty() {
        (x, y, "start")
    } else {
        match label_position {
            "above" => (x, y - size / 2.0 - 4.0, "middle"),
            "below" => (x, y + size / 2.0 + 14.0, "middle"),
            "left" => (x - size / 2.0 - 4.0, y + 4.0, "end"),
            _ => (x + size / 2.0 + 4.0, y + 4.0, "start"),
        }
    };

    let mut svg = String::new();

    // Background rect behind symbol + label for legibility
    if background != "none" {
        let pad = 4.0;
        let half = size / 2.0;
        let font_size = 12.0;
        let label_width = label.chars().count() as f64 * font_size * 0.6;
        let label_height = font_size;

        // Bounding box of symbol
        let (mut min_x, mut min_y, mut max_x, mut max_y) =
            (x - half, y - half, x + half, y + half);

        // Extend to include label
        if !label.is_empty() {
            match label_position {
                "above" => {
                    min_x = min_x.min(lx - label_width / 2.0);
                    max_x = max_x.max(lx + label_width / 2.0);
                    min_y = min_y.min(ly - label_height);
                }
                "below" => {
                    min_x = min_x.min(lx - label_width / 2.0);
                    max_x = max_x.max(lx + label_width / 2.0);
                    max_y = max_y.max(ly + 2.0);
                }
                "left" => {
                    min_x = min_x.min(lx - label_width);
                    min_y = min_y.min(ly - label_height / 2.0);
                    max_y = max_y.max(ly + label_height / 2.0);
                }
                _ => {
                    // right (default)
                    max_x = max_x.max(lx + label_width);
                    min_y = min_y.min(ly - label_height / 2.0);
                    max_y = max_y.max(ly + label_height / 2.0);
                }
            }
        }

        let _ = write!(
            svg,
            r#"<rect x="{}" y="{}" width="{}" height="{}" rx="4" fill="{}" opacity="0.5" stroke="none"/>"#,
            fmt_coord(min_x - pad),
            fmt_coord(min_y - pad),
            fmt_coord(max_x - min_x + pad * 2.0),
            fmt_coord(max_y - min_y + pad * 2.0),
            crate::compile::xml_escape(background),
        );
    }

    let symbol_id = format!("s:marker-{symbol}");

    let _ = write!(
        svg,
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
        svg.push_str(&svg_text(lx, ly, label, anchor, 12, ""));
    }

    svg
}
