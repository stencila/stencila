use super::*;

/// Expand `<s:roi-rect>` into a rectangle outline.
///
/// Supported attributes:
/// - `x`/`y`: top-left position
/// - `width`/`height`: optional explicit dimensions (if using from/to, computed automatically)
/// - `from`/`to`: anchor-based bounds
/// - `label`: optional text label
/// - `label-position`: `above` (default), `below`, `center`, `left`, `right`
/// - `stroke-style`: `solid` (default), `dashed`, `dotted`
pub fn expand_rect(attrs: &Attrs, ctx: &mut ComponentContext) -> String {
    let (x, y, w, h) = if let (Some(x), Some(y)) = (attr_f64(attrs, "x"), attr_f64(attrs, "y")) {
        let w = attr_f64_or(attrs, "width", 50.0);
        let h = attr_f64_or(attrs, "height", 50.0);
        (x, y, w, h)
    } else if let (Some(start), Some(end)) = (
        resolve_position(attrs, "x", "y", Some("from"), "dx", "dy", ctx.anchors),
        resolve_target(attrs, ctx.anchors),
    ) {
        let x = start.0.min(end.0);
        let y = start.1.min(end.1);
        let w = (end.0 - start.0).abs();
        let h = (end.1 - start.1).abs();
        (x, y, w, h)
    } else {
        ctx.messages.push(CompilationMessage::error(
            "<s:roi-rect> requires position and size (x,y,width,height or from/to)",
        ));
        return String::new();
    };

    let label = attr_str(attrs, "label", "");
    let label_position = attr_str(attrs, "label-position", "above");
    let stroke_style = attr_str(attrs, "stroke-style", "solid");
    let pass = pass_through_attrs(attrs);
    let dash = stroke_dash_attr(stroke_style);

    let mut svg = format!(
        r#"<rect x="{}" y="{}" width="{}" height="{}" fill="none" stroke="currentColor"{}{}/>"#,
        fmt_coord(x),
        fmt_coord(y),
        fmt_coord(w),
        fmt_coord(h),
        dash,
        pass
    );

    if !label.is_empty() {
        let (lx, ly) = label_coords(x, y, w, h, label_position);
        svg.push_str(&svg_text(lx, ly, label, "middle", 12, ""));
    }

    svg
}

/// Expand `<s:roi-ellipse>` into an ellipse outline.
///
/// Supported attributes:
/// - `cx`/`cy`: center position
/// - `rx`/`ry`: radii
/// - `label`, `label-position`, `stroke-style`: same as roi-rect
pub fn expand_ellipse(attrs: &Attrs, ctx: &mut ComponentContext) -> String {
    let (Some(cx), Some(cy), Some(rx), Some(ry)) = (
        attr_f64(attrs, "cx"),
        attr_f64(attrs, "cy"),
        attr_f64(attrs, "rx"),
        attr_f64(attrs, "ry"),
    ) else {
        ctx.messages.push(CompilationMessage::error(
            "<s:roi-ellipse> requires 'cx', 'cy', 'rx', and 'ry' attributes",
        ));
        return String::new();
    };

    let label = attr_str(attrs, "label", "");
    let label_position = attr_str(attrs, "label-position", "above");
    let stroke_style = attr_str(attrs, "stroke-style", "solid");
    let pass = pass_through_attrs(attrs);
    let dash = stroke_dash_attr(stroke_style);

    let mut svg = format!(
        r#"<ellipse cx="{}" cy="{}" rx="{}" ry="{}" fill="none" stroke="currentColor"{}{}/>"#,
        fmt_coord(cx),
        fmt_coord(cy),
        fmt_coord(rx),
        fmt_coord(ry),
        dash,
        pass
    );

    if !label.is_empty() {
        let (lx, ly) = label_coords(cx - rx, cy - ry, rx * 2.0, ry * 2.0, label_position);
        svg.push_str(&svg_text(lx, ly, label, "middle", 12, ""));
    }

    svg
}

fn stroke_dash_attr(stroke_style: &str) -> String {
    match stroke_style {
        "dashed" => r#" stroke-dasharray="6 4""#.to_string(),
        "dotted" => r#" stroke-dasharray="2 3""#.to_string(),
        _ => String::new(), // solid
    }
}

fn label_coords(x: f64, y: f64, w: f64, h: f64, position: &str) -> (f64, f64) {
    match position {
        "below" => (x + w / 2.0, y + h + 16.0),
        "center" => (x + w / 2.0, y + h / 2.0),
        "left" => (x - 8.0, y + h / 2.0),
        "right" => (x + w + 8.0, y + h / 2.0),
        _ => (x + w / 2.0, y - 8.0), // above (default)
    }
}
