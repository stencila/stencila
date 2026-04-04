use super::*;

/// Expand `<s:spotlight>` into an inverse-highlight mask.
///
/// Dims everything outside a specified region, drawing attention to the
/// spotlighted area. Uses an SVG `<mask>` element.
///
/// Supported attributes:
/// - `cx`/`cy`: center of the spotlight region
/// - `r`: radius for circular spotlight (default: 50)
/// - `shape`: `circle` (default), `rect`
/// - `width`/`height`: dimensions for rectangular spotlight
/// - `opacity`: opacity of the dimmed area (0.0–1.0, default: 0.6)
/// - `rx`/`ry`: radii for elliptical spotlight (alternative to `r`)
pub fn expand(attrs: &Attrs, ctx: &mut ComponentContext) -> String {
    let (Some(cx), Some(cy)) = (attr_f64(attrs, "cx"), attr_f64(attrs, "cy")) else {
        ctx.messages.push(CompilationMessage::error(
            "<s:spotlight> requires 'cx' and 'cy' attributes",
        ));
        return String::new();
    };

    let shape = attr_str(attrs, "shape", "circle");
    let opacity = attr_f64_or(attrs, "opacity", 0.6);
    let pass = pass_through_attrs(attrs);

    let mut mask_id = format!(
        "s-spotlight-{}-{}-{}",
        fmt_coord(cx),
        fmt_coord(cy),
        fmt_coord(opacity)
    );

    // Build the mask cutout shape (white = visible, black = hidden)
    let cutout = match shape {
        "rect" => {
            let w = attr_f64_or(attrs, "width", 100.0);
            let h = attr_f64_or(attrs, "height", 100.0);
            mask_id.push_str(&format!("-rect-{}-{}", fmt_coord(w), fmt_coord(h)));
            format!(
                r#"<rect x="{}" y="{}" width="{}" height="{}" fill="white"/>"#,
                fmt_coord(cx - w / 2.0),
                fmt_coord(cy - h / 2.0),
                fmt_coord(w),
                fmt_coord(h),
            )
        }
        _ => {
            // circle or ellipse
            let rx = attr_f64(attrs, "rx")
                .or_else(|| attr_f64(attrs, "r"))
                .unwrap_or(50.0);
            let ry = attr_f64(attrs, "ry").unwrap_or(rx);
            mask_id.push_str(&format!("-ellipse-{}-{}", fmt_coord(rx), fmt_coord(ry)));
            format!(
                r#"<ellipse cx="{}" cy="{}" rx="{}" ry="{}" fill="white"/>"#,
                fmt_coord(cx),
                fmt_coord(cy),
                fmt_coord(rx),
                fmt_coord(ry),
            )
        }
    };

    // The mask controls where the dimming overlay is visible:
    // - white areas in the mask = overlay is visible (dimmed)
    // - black areas in the mask = overlay is hidden (spotlight cutout)
    // The overlay rect is black with the user's opacity, masked to exclude the spotlight region.
    format!(
        r#"<mask id="{mask_id}"><rect width="100%" height="100%" fill="white"/>{}</mask><rect width="100%" height="100%" fill="black" opacity="{}" mask="url(#{mask_id})"{pass}/>"#,
        // Invert cutout: use fill="black" so the spotlight region is hidden from the overlay
        cutout.replace("fill=\"white\"", "fill=\"black\""),
        opacity,
    )
}
