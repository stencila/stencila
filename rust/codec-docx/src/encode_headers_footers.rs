use serde_json::Value;
use std::collections::BTreeMap;

use crate::encode_utils::{
    build_paragraph_border, build_tab_stops, get_color_hex, get_font_size_half_points, get_var,
    resolve_border_tokens,
};

/// Build header XML file
///
/// **CSS Tokens Source**: `web/src/themes/base/pages.css`
///
/// **Tokens Applied**:
/// - Content positioning (left/center/right via tab stops)
/// - `--page-margin-font-family` → w:rFonts
/// - `--page-margin-font-size` → w:sz/w:szCs
/// - `--page-margin-color` → w:color
/// - `--page-top-border-*` tokens → w:pBdr bottom border (hierarchical resolution)
///
/// # Arguments
/// * `vars` - Pre-computed theme variables
/// * `left_content` - Content for left-aligned position (token name)
/// * `center_content` - Content for center-aligned position (token name)
/// * `right_content` - Content for right-aligned position (token name)
/// * `page_width` - Page content width in twips for tab stop positioning
///
/// # Returns
/// Complete header XML file content, or None if no content defined
pub(crate) fn build_header_xml(
    vars: &BTreeMap<String, Value>,
    left_content: &str,
    center_content: &str,
    right_content: &str,
    page_width: u32,
) -> Option<String> {
    let left = get_var(vars, left_content).unwrap_or_default();
    let center = get_var(vars, center_content).unwrap_or_default();
    let right = get_var(vars, right_content).unwrap_or_default();

    // Skip if all content is empty
    if left.is_empty() && center.is_empty() && right.is_empty() {
        return None;
    }

    let mut xml = String::with_capacity(2048);

    xml.push_str(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:hdr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
<w:p><w:pPr>"#,
    );

    // Add tab stops for left/center/right positioning
    xml.push_str(&build_tab_stops(page_width));

    // Add border if defined (hierarchical: page-top-border → page-horizontal-border → page-border)
    if let Some((width, color, style)) = resolve_border_tokens(
        vars,
        "page-top-border",
        Some("page-horizontal-border"),
        "page-border",
    ) {
        xml.push_str(&build_paragraph_border("bottom", &width, &color, &style));
    }

    xml.push_str("</w:pPr>");

    // Build character properties (font, size, color)
    let mut char_props = String::new();

    if let Some(font) = get_var(vars, "page-margin-font-family") {
        char_props.push_str(&format!(
            r#"<w:rFonts w:ascii="{font}" w:hAnsi="{font}" w:eastAsia="{font}" w:cs=""/>"#
        ));
    }

    if let Some(color) = get_color_hex(vars, "page-margin-color") {
        char_props.push_str(&format!(r#"<w:color w:val="{color}"/>"#));
    }

    if let Some(size) = get_font_size_half_points(vars, "page-margin-font-size") {
        char_props.push_str(&format!(
            r#"<w:sz w:val="{size}"/><w:szCs w:val="{size}"/>"#
        ));
    }

    // Left content
    if !left.is_empty() {
        xml.push_str(&format!(
            r#"<w:r><w:rPr>{char_props}</w:rPr><w:t xml:space="preserve">{left}</w:t></w:r>"#
        ));
    }

    // Tab to center
    if !center.is_empty() {
        xml.push_str(&format!(
            r#"<w:r><w:rPr>{char_props}</w:rPr><w:tab/></w:r><w:r><w:rPr>{char_props}</w:rPr><w:t xml:space="preserve">{center}</w:t></w:r>"#
        ));
    }

    // Tab to right
    if !right.is_empty() {
        xml.push_str(&format!(
            r#"<w:r><w:rPr>{char_props}</w:rPr><w:tab/></w:r><w:r><w:rPr>{char_props}</w:rPr><w:t xml:space="preserve">{right}</w:t></w:r>"#
        ));
    }

    xml.push_str("</w:p></w:hdr>");

    Some(xml)
}

/// Build footer XML file
///
/// **CSS Tokens Source**: `web/src/themes/base/pages.css`
///
/// **Tokens Applied**:
/// - Content positioning (left/center/right via tab stops)
/// - `--page-margin-font-family` → w:rFonts
/// - `--page-margin-font-size` → w:sz/w:szCs
/// - `--page-margin-color` → w:color
/// - `--page-bottom-border-*` tokens → w:pBdr top border (hierarchical resolution)
///
/// # Arguments
/// * `vars` - Pre-computed theme variables
/// * `left_content` - Content for left-aligned position (token name)
/// * `center_content` - Content for center-aligned position (token name)
/// * `right_content` - Content for right-aligned position (token name)
/// * `page_width` - Page content width in twips for tab stop positioning
///
/// # Returns
/// Complete footer XML file content, or None if no content defined
pub(crate) fn build_footer_xml(
    vars: &BTreeMap<String, Value>,
    left_content: &str,
    center_content: &str,
    right_content: &str,
    page_width: u32,
) -> Option<String> {
    let left = get_var(vars, left_content).unwrap_or_default();
    let center = get_var(vars, center_content).unwrap_or_default();
    let right = get_var(vars, right_content).unwrap_or_default();

    // Skip if all content is empty
    if left.is_empty() && center.is_empty() && right.is_empty() {
        return None;
    }

    let mut xml = String::with_capacity(2048);

    xml.push_str(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:ftr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
<w:p><w:pPr>"#,
    );

    // Add tab stops for left/center/right positioning
    xml.push_str(&build_tab_stops(page_width));

    // Add border if defined (hierarchical: page-bottom-border → page-horizontal-border → page-border)
    if let Some((width, color, style)) = resolve_border_tokens(
        vars,
        "page-bottom-border",
        Some("page-horizontal-border"),
        "page-border",
    ) {
        xml.push_str(&build_paragraph_border("top", &width, &color, &style));
    }

    xml.push_str("</w:pPr>");

    // Build character properties (font, size, color)
    let mut char_props = String::new();

    if let Some(font) = get_var(vars, "page-margin-font-family") {
        char_props.push_str(&format!(
            r#"<w:rFonts w:ascii="{font}" w:hAnsi="{font}" w:eastAsia="{font}" w:cs=""/>"#
        ));
    }

    if let Some(color) = get_color_hex(vars, "page-margin-color") {
        char_props.push_str(&format!(r#"<w:color w:val="{color}"/>"#));
    }

    if let Some(size) = get_font_size_half_points(vars, "page-margin-font-size") {
        char_props.push_str(&format!(
            r#"<w:sz w:val="{size}"/><w:szCs w:val="{size}"/>"#
        ));
    }

    // Left content
    if !left.is_empty() {
        xml.push_str(&format!(
            r#"<w:r><w:rPr>{char_props}</w:rPr><w:t xml:space="preserve">{left}</w:t></w:r>"#
        ));
    }

    // Tab to center
    if !center.is_empty() {
        xml.push_str(&format!(
            r#"<w:r><w:rPr>{char_props}</w:rPr><w:tab/></w:r><w:r><w:rPr>{char_props}</w:rPr><w:t xml:space="preserve">{center}</w:t></w:r>"#
        ));
    }

    // Tab to right
    if !right.is_empty() {
        xml.push_str(&format!(
            r#"<w:r><w:rPr>{char_props}</w:rPr><w:tab/></w:r><w:r><w:rPr>{char_props}</w:rPr><w:t xml:space="preserve">{right}</w:t></w:r>"#
        ));
    }

    xml.push_str("</w:p></w:ftr>");

    Some(xml)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_resolve_border_tokens_specific() {
        let mut vars = BTreeMap::new();
        vars.insert("page-top-border-width".to_string(), json!(60.0));
        vars.insert("page-top-border-color".to_string(), json!("#FF0000"));
        vars.insert("page-top-border-style".to_string(), json!("solid"));

        let border = resolve_border_tokens(
            &vars,
            "page-top-border",
            Some("page-horizontal-border"),
            "page-border",
        );

        assert!(border.is_some());
        if let Some((width, color, style)) = border {
            assert_eq!(width, "60");
            assert_eq!(color, "FF0000");
            assert_eq!(style, "solid");
        }
    }

    #[test]
    fn test_resolve_border_tokens_hierarchical() {
        let mut vars = BTreeMap::new();
        // No specific top border, but horizontal border defined
        vars.insert("page-horizontal-border-width".to_string(), json!(40.0));
        vars.insert("page-horizontal-border-color".to_string(), json!("#00FF00"));
        vars.insert("page-horizontal-border-style".to_string(), json!("dashed"));

        let border = resolve_border_tokens(
            &vars,
            "page-top-border",
            Some("page-horizontal-border"),
            "page-border",
        );

        assert!(border.is_some());
        if let Some((width, color, style)) = border {
            assert_eq!(width, "40");
            assert_eq!(color, "00FF00");
            assert_eq!(style, "dashed");
        }
    }

    #[test]
    fn test_build_header_xml_with_content() {
        let mut vars = BTreeMap::new();
        vars.insert("page-top-left-content".to_string(), json!("Left"));
        vars.insert("page-top-center-content".to_string(), json!("Center"));
        vars.insert("page-top-right-content".to_string(), json!("Right"));
        vars.insert("page-margin-font-family".to_string(), json!("Arial"));
        vars.insert("page-margin-font-size".to_string(), json!(160.0)); // 8pt in twips
        vars.insert("page-margin-color".to_string(), json!("#333333"));

        let xml = build_header_xml(
            &vars,
            "page-top-left-content",
            "page-top-center-content",
            "page-top-right-content",
            9000,
        );

        assert!(xml.is_some());
        if let Some(xml) = xml {
            assert!(xml.contains("<w:hdr"));
            assert!(xml.contains("Left"));
            assert!(xml.contains("Center"));
            assert!(xml.contains("Right"));
            assert!(xml.contains("Arial"));
            assert!(xml.contains("w:val=\"16\"")); // 8pt = 16 half-points
            assert!(xml.contains("333333"));
        }
    }

    #[test]
    fn test_build_header_xml_empty() {
        let vars = BTreeMap::new();

        let xml = build_header_xml(
            &vars,
            "page-top-left-content",
            "page-top-center-content",
            "page-top-right-content",
            9000,
        );

        assert!(xml.is_none());
    }

    #[test]
    fn test_build_footer_xml_with_border() {
        let mut vars = BTreeMap::new();
        vars.insert("page-bottom-center-content".to_string(), json!("Page"));
        vars.insert("page-bottom-border-width".to_string(), json!(40.0));
        vars.insert("page-bottom-border-color".to_string(), json!("#000000"));
        vars.insert("page-bottom-border-style".to_string(), json!("single"));

        let xml = build_footer_xml(
            &vars,
            "page-bottom-left-content",
            "page-bottom-center-content",
            "page-bottom-right-content",
            9000,
        );

        assert!(xml.is_some());
        if let Some(xml) = xml {
            assert!(xml.contains("<w:ftr"));
            assert!(xml.contains("Page"));
            assert!(xml.contains("<w:pBdr><w:top"));
            assert!(xml.contains("000000"));
        }
    }
}
