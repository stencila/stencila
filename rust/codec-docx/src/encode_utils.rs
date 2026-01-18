use serde_json::Value;
use std::collections::BTreeMap;

// ============================================================================
// Value Extraction Utilities
// ============================================================================

/// Get a string value from computed theme variables
pub(crate) fn get_var(vars: &BTreeMap<String, Value>, name: &str) -> Option<String> {
    vars.get(name).and_then(|v| match v {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    })
}

/// Get a hex color value (strips # prefix for DOCX compatibility)
pub(crate) fn get_color_hex(vars: &BTreeMap<String, Value>, name: &str) -> Option<String> {
    get_var(vars, name).map(|color| color.trim_start_matches('#').to_string())
}

/// Get font size in half-points from a twips variable
pub(crate) fn get_font_size_half_points(
    vars: &BTreeMap<String, Value>,
    name: &str,
) -> Option<String> {
    vars.get(name)
        .and_then(|v| v.as_f64().map(twips_to_half_points))
}

/// Get spacing value in twips as a string
pub(crate) fn get_twips(vars: &BTreeMap<String, Value>, name: &str) -> Option<String> {
    vars.get(name)
        .and_then(|v| v.as_f64().map(|twips| twips.round().to_string()))
}

/// Get spacing value in twips as u32
pub(crate) fn get_twips_u32(vars: &BTreeMap<String, Value>, name: &str) -> Option<u32> {
    vars.get(name)
        .and_then(|v| v.as_f64().map(|twips| twips.round() as u32))
}

/// Get font-variant XML element based on CSS font-variant value
pub(crate) fn get_font_variant_element(vars: &BTreeMap<String, Value>, name: &str) -> String {
    get_var(vars, name)
        .and_then(|variant| {
            let variant = variant.trim();
            match variant {
                "small-caps" => Some(r#"<w:smallCaps/>"#.to_string()),
                "all-small-caps" | "all-caps" => Some(r#"<w:caps/>"#.to_string()),
                _ => None,
            }
        })
        .unwrap_or_default()
}

/// Get text alignment value for DOCX (w:jc)
///
/// Converts CSS text-align values to DOCX justification values.
/// Returns "left" as default if the token is not found or has an unsupported value.
pub(crate) fn get_text_align(vars: &BTreeMap<String, Value>, name: &str) -> String {
    get_var(vars, name)
        .and_then(|align| {
            let align = align.trim();
            match align {
                "left" | "start" => Some("left".to_string()),
                "right" | "end" => Some("right".to_string()),
                "center" => Some("center".to_string()),
                "justify" => Some("both".to_string()),
                _ => None,
            }
        })
        .unwrap_or_else(|| "left".to_string())
}

/// Check if font weight indicates bold (>= 600)
pub(crate) fn is_bold(vars: &BTreeMap<String, Value>, name: &str) -> bool {
    vars.get(name)
        .and_then(|v| v.as_f64())
        .map(|weight| weight >= 600.0)
        .unwrap_or(false)
}

/// Check if font style is italic
pub(crate) fn is_italic(vars: &BTreeMap<String, Value>, name: &str) -> bool {
    get_var(vars, name)
        .map(|style| style.trim() == "italic")
        .unwrap_or(false)
}

/// Extract URL from CSS url() syntax
///
/// Handles various formats:
/// - `url(https://example.com/image.png)`
/// - `url("https://example.com/image.png")`
/// - `url('https://example.com/image.png')`
/// - `url( https://example.com/image.png )` (with whitespace)
///
/// Returns None if the value is not a valid url() expression.
pub(crate) fn extract_url(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if let Some(url) = trimmed.strip_prefix("url(") {
        url.strip_suffix(')')
            .map(|s| s.trim().trim_matches(|c| c == '"' || c == '\''))
            .map(String::from)
    } else {
        None
    }
}

// ============================================================================
// Conversion Utilities
// ============================================================================

/// Convert twips to half-points (DOCX font size unit: 1pt = 2 half-points, 1pt = 20 twips)
pub(crate) fn twips_to_half_points(twips: f64) -> String {
    (twips / 10.0).round().to_string()
}

/// Convert border width from twips to eighths of a point for DOCX w:sz attribute
///
/// DOCX border sizes use eighths of a point, while our theme uses twips.
/// Conversion: 1 twip = 0.4 eighths of a point
pub(crate) fn twips_to_border_size(twips: &str) -> u32 {
    twips
        .parse::<f64>()
        .map(|t| (t * 0.4).round() as u32)
        .unwrap_or(0)
}

/// Convert a page size token to dimensions in twips
///
/// # Arguments
/// * `page_size` - Page size identifier ("a4", "letter", etc.)
///
/// # Returns
/// Tuple of (width, height) in twips, or None if size is unknown
pub(crate) fn page_size_to_twips(page_size: &str) -> Option<(u32, u32)> {
    match page_size.to_lowercase().as_str() {
        // A4: 210mm × 297mm
        "a4" => Some((11906, 16838)),
        // Letter: 8.5in × 11in
        "letter" | "us-letter" => Some((12240, 15840)),
        // Legal: 8.5in × 14in
        "legal" | "us-legal" => Some((12240, 20160)),
        // A3: 297mm × 420mm
        "a3" => Some((16838, 23811)),
        // A5: 148mm × 210mm
        "a5" => Some((8391, 11906)),
        // Tabloid: 11in × 17in
        "tabloid" => Some((15840, 24480)),
        _ => None,
    }
}

// ============================================================================
// Border Utilities
// ============================================================================

/// Get border value for w:val attribute based on CSS border style
pub(crate) fn get_border_val(style: &str) -> &str {
    match style {
        "dashed" => "dashed",
        "dotted" => "dotted",
        "double" => "double",
        _ => "single", // solid and others default to single
    }
}

/// Resolve border tokens from computed theme variables
///
/// Generates a border if any of the *-full, *-wide, or *-narrow tokens are defined.
///
/// Since CSS variables are already resolved by `computed_variables_with_overrides`,
/// the hierarchical fallback (specific → horizontal → general) has already been
/// applied at the CSS level. We only need to check the specific prefix.
///
/// Returns (width, color, style) tuple or None if no border defined
pub(crate) fn resolve_border_tokens(
    vars: &BTreeMap<String, Value>,
    prefix: &str,
) -> Option<(String, String, String)> {
    if let Some(width) = get_twips(vars, &format!("{prefix}-full"))
        .or_else(|| get_twips(vars, &format!("{prefix}-wide")))
        .or_else(|| get_twips(vars, &format!("{prefix}-narrow")))
    {
        let width_num = width.parse::<f64>().unwrap_or(0.0);
        if width_num > 0.0 {
            let color = get_color_hex(vars, &format!("{prefix}-color"))
                .unwrap_or_else(|| "000000".to_string());
            let style =
                get_var(vars, &format!("{prefix}-style")).unwrap_or_else(|| "solid".to_string());
            return Some((width, color, style));
        }
    }
    None
}

// ============================================================================
// XML Building Utilities
// ============================================================================

/// Escape XML.
///
/// Replaces the five XML-sensitive characters with their corresponding
/// entity references.
///
/// | character | entity  |
/// |-----------|---------|
/// | `&`       | `&amp;` |
/// | `<`       | `&lt;`  |
/// | `>`       | `&gt;`  |
/// | `"`       | `&quot;`|
/// | `'`       | `&apos;`|
pub(crate) fn escape_xml(input: &str) -> String {
    // Pre-allocate slightly more than the input length to avoid
    // frequent reallocations for typical "few escapables" cases.
    let mut out = String::with_capacity(input.len() + 8);

    for ch in input.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(ch),
        }
    }

    out
}

/// Build <w:rFonts> element from font-family variable
///
/// The font family value should already be a resolved font name (not a CSS stack)
/// since fonts are resolved earlier in the encoding process.
pub(crate) fn build_font_element(vars: &BTreeMap<String, Value>, var_name: &str) -> String {
    get_var(vars, var_name)
        .map(|family| {
            format!(r#"<w:rFonts w:ascii="{family}" w:hAnsi="{family}" w:eastAsia="{family}" w:cs="" />"#)
        })
        .unwrap_or_default()
}

/// Build <w:sz> and <w:szCs> elements for font size
pub(crate) fn build_size_elements(half_points: &str) -> String {
    format!(r#"<w:sz w:val="{half_points}"/><w:szCs w:val="{half_points}"/>"#)
}

/// Build <w:color> element from hex color (without # prefix)
pub(crate) fn build_color_element(hex: &str) -> String {
    format!(r#"<w:color w:val="{hex}"/>"#)
}

/// Build <w:spacing> element with before/after attributes
pub(crate) fn build_spacing_element(
    before_twips: Option<&str>,
    after_twips: Option<&str>,
) -> String {
    let mut attrs = String::new();

    if let Some(before) = before_twips {
        attrs.push_str(&format!(r#" w:before="{before}""#));
    }
    if let Some(after) = after_twips {
        attrs.push_str(&format!(r#" w:after="{after}""#));
    }

    if attrs.is_empty() {
        String::new()
    } else {
        format!("<w:spacing{attrs}/>")
    }
}

/// Build <w:shd> element for paragraph background color
///
/// Maps to CSS background-color property. Uses "clear" pattern (no overlay)
/// with the specified fill color.
///
/// # Arguments
/// * `hex` - Background color in hex format without # prefix (e.g., "F0F0F0")
pub(crate) fn build_paragraph_shading_element(hex: &str) -> String {
    format!(r#"<w:shd w:val="clear" w:color="auto" w:fill="{hex}"/>"#)
}

/// Build <w:pBdr> element with left border only
///
/// Maps to CSS border-left property. Commonly used for quote blocks and callouts.
///
/// # Arguments
/// * `width_twips` - Border width in twips (already converted by theme processor)
/// * `color_hex` - Border color in hex format without # prefix
pub(crate) fn build_paragraph_left_border_element(width_twips: &str, color_hex: &str) -> String {
    // Convert twips to eighths of a point for w:sz (1 twip = 0.4 eighths-pt)
    let sz = width_twips
        .parse::<f64>()
        .map(|twips| (twips * 0.4).round() as u32)
        .unwrap_or(0);

    format!(
        r#"<w:pBdr><w:left w:val="single" w:sz="{sz}" w:space="0" w:color="{color_hex}"/></w:pBdr>"#
    )
}

/// Build <w:pBdr> element with bottom border
///
/// Maps to CSS border-bottom property. Commonly used for headings and section dividers.
///
/// # Arguments
/// * `width_twips` - Border width in twips (already converted by theme processor)
/// * `color_hex` - Border color in hex format without # prefix
/// * `style` - CSS border style (e.g., "solid", "dashed", "dotted")
/// * `padding_twips` - Optional padding between text and border in twips (defaults to 0)
pub(crate) fn build_paragraph_bottom_border_element(
    width_twips: &str,
    color_hex: &str,
    style: &str,
    padding_twips: Option<&str>,
) -> String {
    // Convert twips to eighths of a point for w:sz (1 twip = 0.4 eighths-pt)
    let sz = twips_to_border_size(width_twips);

    // Convert padding from twips to points for w:space (1 point = 20 twips)
    let space = padding_twips
        .and_then(|p| p.parse::<f64>().ok())
        .map(|twips| (twips / 20.0).round() as u32)
        .unwrap_or(0);

    let val = get_border_val(style);

    format!(
        r#"<w:pBdr><w:bottom w:val="{val}" w:sz="{sz}" w:space="{space}" w:color="{color_hex}"/></w:pBdr>"#
    )
}

/// Build paragraph border element for header/footer
///
/// # Arguments
/// * `edge` - Border edge ("top" or "bottom")
/// * `width_twips` - Border width in twips
/// * `color_hex` - Border color in hex without # prefix
/// * `style` - CSS border style
pub(crate) fn build_paragraph_border(
    edge: &str,
    width_twips: &str,
    color_hex: &str,
    style: &str,
) -> String {
    let sz = twips_to_border_size(width_twips);
    let val = get_border_val(style);

    format!(
        r#"<w:pBdr><w:{edge} w:val="{val}" w:sz="{sz}" w:space="0" w:color="{color_hex}"/></w:pBdr>"#
    )
}

/// Build table cell borders element (w:tcBorders)
///
/// # Arguments
/// * `vars` - Theme variables
/// * `top_tokens` - Optional (width, color, style) token names for top border
/// * `bottom_tokens` - Optional (width, color, style) token names for bottom border
/// * `left_tokens` - Optional (width, color, style) token names for left border
/// * `right_tokens` - Optional (width, color, style) token names for right border
pub(crate) fn build_cell_borders_element(
    vars: &BTreeMap<String, Value>,
    top_tokens: Option<(&str, &str, &str)>,
    bottom_tokens: Option<(&str, &str, &str)>,
    left_tokens: Option<(&str, &str, &str)>,
    right_tokens: Option<(&str, &str, &str)>,
) -> String {
    let mut borders = Vec::new();

    // Helper to build a single border
    let build_border = |edge: &str, width_token: &str, color_token: &str, style_token: &str| {
        if let Some(width) = get_twips(vars, width_token) {
            let width_num = width.parse::<f64>().unwrap_or(0.0);
            if width_num > 0.0 {
                let sz = twips_to_border_size(&width);
                let color =
                    get_color_hex(vars, color_token).unwrap_or_else(|| "000000".to_string());
                let style = get_var(vars, style_token).unwrap_or_else(|| "solid".to_string());
                let val = get_border_val(&style);
                return Some(format!(
                    r#"<w:{edge} w:val="{val}" w:sz="{sz}" w:space="0" w:color="{color}"/>"#
                ));
            }
        }
        None
    };

    if let Some((w, c, s)) = top_tokens
        && let Some(border) = build_border("top", w, c, s)
    {
        borders.push(border);
    }
    if let Some((w, c, s)) = bottom_tokens
        && let Some(border) = build_border("bottom", w, c, s)
    {
        borders.push(border);
    }
    if let Some((w, c, s)) = left_tokens
        && let Some(border) = build_border("left", w, c, s)
    {
        borders.push(border);
    }
    if let Some((w, c, s)) = right_tokens
        && let Some(border) = build_border("right", w, c, s)
    {
        borders.push(border);
    }

    if borders.is_empty() {
        String::new()
    } else {
        format!("<w:tcBorders>{}</w:tcBorders>", borders.join(""))
    }
}

/// Build table borders element (w:tblBorders) for outer table borders
///
/// # Arguments
/// * `vars` - Theme variables
/// * `top_tokens` - (width, color, style) token names for top border
/// * `bottom_tokens` - (width, color, style) token names for bottom border
/// * `left_tokens` - (width, color, style) token names for left border
/// * `right_tokens` - (width, color, style) token names for right border
pub(crate) fn build_table_borders_element(
    vars: &BTreeMap<String, Value>,
    top_tokens: (&str, &str, &str),
    bottom_tokens: (&str, &str, &str),
    left_tokens: (&str, &str, &str),
    right_tokens: (&str, &str, &str),
) -> String {
    let mut borders = Vec::new();

    // Helper to build a single border
    let build_border = |edge: &str, width_token: &str, color_token: &str, style_token: &str| {
        if let Some(width) = get_twips(vars, width_token) {
            let width_num = width.parse::<f64>().unwrap_or(0.0);
            if width_num > 0.0 {
                let sz = twips_to_border_size(&width);
                let color =
                    get_color_hex(vars, color_token).unwrap_or_else(|| "000000".to_string());
                let style = get_var(vars, style_token).unwrap_or_else(|| "solid".to_string());
                let val = get_border_val(&style);
                return Some(format!(
                    r#"<w:{edge} w:val="{val}" w:sz="{sz}" w:space="0" w:color="{color}"/>"#
                ));
            }
        }
        None
    };

    let (w, c, s) = top_tokens;
    if let Some(border) = build_border("top", w, c, s) {
        borders.push(border);
    }

    let (w, c, s) = bottom_tokens;
    if let Some(border) = build_border("bottom", w, c, s) {
        borders.push(border);
    }

    let (w, c, s) = left_tokens;
    if let Some(border) = build_border("left", w, c, s) {
        borders.push(border);
    }

    let (w, c, s) = right_tokens;
    if let Some(border) = build_border("right", w, c, s) {
        borders.push(border);
    }

    if borders.is_empty() {
        String::new()
    } else {
        format!("<w:tblBorders>{}</w:tblBorders>", borders.join(""))
    }
}

/// Build tab stops for left/center/right positioning in header/footer
///
/// Creates tab stops based on which content positions are defined:
/// - If center content exists: Center tab at 50% + Right tab at 100%
/// - If no center content: Only Right tab at 100% (allows right content more space)
///
/// This dynamic approach prevents long right-aligned content (like document titles)
/// from wrapping when there's no center content competing for space.
///
/// # Arguments
/// * `page_width` - Page width in twips (from page-content-width or calculated)
/// * `has_center` - Whether center content is defined
pub(crate) fn build_tab_stops(page_width: u32, has_center: bool) -> String {
    let right_pos = page_width;

    if has_center {
        let center_pos = page_width / 2;
        format!(
            r#"<w:tabs><w:tab w:val="center" w:pos="{center_pos}"/><w:tab w:val="right" w:pos="{right_pos}"/></w:tabs>"#
        )
    } else {
        format!(r#"<w:tabs><w:tab w:val="right" w:pos="{right_pos}"/></w:tabs>"#)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_get_var() {
        let mut vars = BTreeMap::new();
        vars.insert("test-string".to_string(), json!("hello"));
        vars.insert("test-number".to_string(), json!(42));

        assert_eq!(get_var(&vars, "test-string"), Some("hello".to_string()));
        assert_eq!(get_var(&vars, "test-number"), Some("42".to_string()));
        assert_eq!(get_var(&vars, "nonexistent"), None);
    }

    #[test]
    fn test_get_color_hex() {
        let mut vars = BTreeMap::new();
        vars.insert("color1".to_string(), json!("#FF0000"));
        vars.insert("color2".to_string(), json!("00FF00"));

        assert_eq!(get_color_hex(&vars, "color1"), Some("FF0000".to_string()));
        assert_eq!(get_color_hex(&vars, "color2"), Some("00FF00".to_string()));
    }

    #[test]
    fn test_twips_to_half_points() {
        assert_eq!(twips_to_half_points(240.0), "24"); // 12pt = 240 twips = 24 half-points
        assert_eq!(twips_to_half_points(320.0), "32"); // 16pt = 320 twips = 32 half-points
    }

    #[test]
    fn test_twips_to_border_size() {
        assert_eq!(twips_to_border_size("40"), 16); // 40 twips = 16 eighths of a point
        assert_eq!(twips_to_border_size("60"), 24); // 60 twips = 24 eighths of a point
    }

    #[test]
    fn test_page_size_to_twips() {
        assert_eq!(page_size_to_twips("a4"), Some((11906, 16838)));
        assert_eq!(page_size_to_twips("letter"), Some((12240, 15840)));
        assert_eq!(page_size_to_twips("unknown"), None);
    }

    #[test]
    fn test_get_border_val() {
        assert_eq!(get_border_val("dashed"), "dashed");
        assert_eq!(get_border_val("dotted"), "dotted");
        assert_eq!(get_border_val("double"), "double");
        assert_eq!(get_border_val("solid"), "single");
        assert_eq!(get_border_val("unknown"), "single");
    }

    #[test]
    fn test_is_bold() {
        let mut vars = BTreeMap::new();
        vars.insert("bold-weight".to_string(), json!(700.0));
        vars.insert("normal-weight".to_string(), json!(400.0));

        assert!(is_bold(&vars, "bold-weight"));
        assert!(!is_bold(&vars, "normal-weight"));
    }

    #[test]
    fn test_is_italic() {
        let mut vars = BTreeMap::new();
        vars.insert("italic-style".to_string(), json!("italic"));
        vars.insert("normal-style".to_string(), json!("normal"));

        assert!(is_italic(&vars, "italic-style"));
        assert!(!is_italic(&vars, "normal-style"));
    }

    #[test]
    fn test_escape_xml() {
        assert_eq!(escape_xml("Test & Co."), "Test &amp; Co.");
        assert_eq!(escape_xml("<tag>"), "&lt;tag&gt;");
        assert_eq!(escape_xml("'quote'"), "&apos;quote&apos;");
    }

    #[test]
    fn test_get_text_align() {
        let mut vars = BTreeMap::new();
        vars.insert("align-left".to_string(), json!("left"));
        vars.insert("align-center".to_string(), json!("center"));
        vars.insert("align-right".to_string(), json!("right"));
        vars.insert("align-justify".to_string(), json!("justify"));

        assert_eq!(get_text_align(&vars, "align-left"), "left");
        assert_eq!(get_text_align(&vars, "align-center"), "center");
        assert_eq!(get_text_align(&vars, "align-right"), "right");
        assert_eq!(get_text_align(&vars, "align-justify"), "both");
        assert_eq!(get_text_align(&vars, "nonexistent"), "left");
    }

    #[test]
    fn test_extract_url_basic() {
        assert_eq!(
            extract_url("url(https://example.com/image.png)"),
            Some("https://example.com/image.png".to_string())
        );
    }

    #[test]
    fn test_extract_url_with_double_quotes() {
        assert_eq!(
            extract_url(r#"url("https://example.com/image.png")"#),
            Some("https://example.com/image.png".to_string())
        );
    }

    #[test]
    fn test_extract_url_with_single_quotes() {
        assert_eq!(
            extract_url("url('https://example.com/image.png')"),
            Some("https://example.com/image.png".to_string())
        );
    }

    #[test]
    fn test_extract_url_with_whitespace() {
        assert_eq!(
            extract_url("url( https://example.com/image.png )"),
            Some("https://example.com/image.png".to_string())
        );
        assert_eq!(
            extract_url("url( \"https://example.com/image.png\" )"),
            Some("https://example.com/image.png".to_string())
        );
    }

    #[test]
    fn test_extract_url_with_query_params() {
        assert_eq!(
            extract_url("url(https://placehold.co/60x30/white/blue?text=Stencila)"),
            Some("https://placehold.co/60x30/white/blue?text=Stencila".to_string())
        );
    }

    #[test]
    fn test_extract_url_not_url() {
        assert_eq!(extract_url("https://example.com"), None);
        assert_eq!(extract_url("some text"), None);
        assert_eq!(extract_url(""), None);
    }

    #[test]
    fn test_extract_url_malformed() {
        assert_eq!(extract_url("url(https://example.com"), None); // Missing closing paren
        assert_eq!(extract_url("https://example.com)"), None); // Missing url(
    }
}
