use super::{
    Attrs, CompilationMessage, ComponentContext, attr_f64_or, attr_str, pass_through_attrs,
    resolve_position, resolve_stroke, resolve_text, svg_line, svg_text,
};

/// Expand `<s:compass>` into a directional compass rose.
///
/// Supported attributes:
/// - `x`/`y` or `at`: center position
/// - `size`: overall size in viewBox units, defaults to 50
/// - `variant`: `arrow` (default single-axis arrow) or `full` (four-axis cross)
/// - `axes`: space-separated axis pairs like `"N/S E/W"` or `"A/P D/V"`
///   - Default: `"N/S E/W"`
///   - For `variant="arrow"`: only the positive label of the first axis is shown
///   - For `variant="full"`: all four labels are shown at cardinal positions
/// - `label`: optional additional text label
pub fn expand(attrs: &Attrs, ctx: &mut ComponentContext) -> String {
    let Some((x, y)) = resolve_position(attrs, "x", "y", Some("at"), "dx", "dy", ctx.anchors)
    else {
        ctx.messages.push(CompilationMessage::error(
            "<s:compass> requires position (x,y or at)",
        ));
        return String::new();
    };

    let size = attr_f64_or(attrs, "size", 50.0);
    let variant = attr_str(attrs, "variant", "arrow");
    let axes_str = attr_str(attrs, "axes", "N/S E/W");
    let pass = pass_through_attrs(attrs);
    let stroke = resolve_stroke(attrs);
    let text_fill = resolve_text(attrs);

    let axes = parse_axes(axes_str);
    let r = size / 2.0;

    let mut svg = String::new();

    if variant == "full" {
        // Primary axis: up/down (first pair)
        svg.push_str(&svg_line(x, y - r, x, y + r, stroke, &pass));
        // Secondary axis: left/right (second pair)
        svg.push_str(&svg_line(x - r, y, x + r, y, stroke, ""));

        // Labels at cardinal positions
        if let Some(up) = axes.first().map(|(p, _)| p.as_str()) {
            svg.push_str(&svg_text(
                x,
                y - r - 4.0,
                up,
                "middle",
                12,
                text_fill,
                r#" font-weight="bold""#,
            ));
        }
        if let Some(down) = axes.first().map(|(_, n)| n.as_str()) {
            svg.push_str(&svg_text(x, y + r + 14.0, down, "middle", 12, text_fill, ""));
        }
        if let Some(right) = axes.get(1).map(|(p, _)| p.as_str()) {
            svg.push_str(&svg_text(x + r + 4.0, y + 4.0, right, "start", 12, text_fill, ""));
        }
        if let Some(left) = axes.get(1).map(|(_, n)| n.as_str()) {
            svg.push_str(&svg_text(x - r - 4.0, y + 4.0, left, "end", 12, text_fill, ""));
        }
    } else {
        // arrow variant: single directional arrow pointing up with label
        svg.push_str(&svg_line(
            x,
            y - r,
            x,
            y + r * 0.3,
            stroke,
            &format!(r#" marker-start="url(#s:arrow-closed)"{pass}"#),
        ));

        if let Some(label) = axes.first().map(|(p, _)| p.as_str()) {
            svg.push_str(&svg_text(
                x,
                y - r - 4.0,
                label,
                "middle",
                12,
                text_fill,
                r#" font-weight="bold""#,
            ));
        }
    }

    svg
}

/// Parse axes string like "N/S E/W" or "A/P D/V" into pairs of (positive, negative).
fn parse_axes(axes_str: &str) -> Vec<(String, String)> {
    axes_str
        .split_whitespace()
        .filter_map(|pair| {
            let mut parts = pair.split('/');
            let positive = parts.next()?.to_string();
            let negative = parts.next()?.to_string();
            Some((positive, negative))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_default_axes() {
        let axes = parse_axes("N/S E/W");
        assert_eq!(axes.len(), 2);
        assert_eq!(axes[0], ("N".to_string(), "S".to_string()));
        assert_eq!(axes[1], ("E".to_string(), "W".to_string()));
    }

    #[test]
    fn parse_anatomical_axes() {
        let axes = parse_axes("A/P D/V");
        assert_eq!(axes.len(), 2);
        assert_eq!(axes[0], ("A".to_string(), "P".to_string()));
        assert_eq!(axes[1], ("D".to_string(), "V".to_string()));
    }
}
