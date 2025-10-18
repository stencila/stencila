use serde_json::Value;
use std::collections::BTreeMap;

use crate::encode_utils::{get_twips_u32, get_var, page_size_to_twips};

/// Build the w:sectPr (section properties) element with page layout from theme
///
/// # Arguments
/// * `vars` - Pre-computed theme variables with page layout tokens
/// * `has_header` - Whether header content exists (adds headerReference if true)
/// * `has_footer` - Whether footer content exists (adds footerReference if true)
/// * `has_first_header` - Whether first page header differs from regular pages
/// * `has_first_footer` - Whether first page footer differs from regular pages
///
/// # Returns
/// XML string for w:sectPr element
pub(crate) fn build_section_properties(
    vars: &BTreeMap<String, Value>,
    has_header: bool,
    has_footer: bool,
    has_first_header: bool,
    has_first_footer: bool,
) -> String {
    let mut xml = String::with_capacity(1024);

    xml.push_str("<w:sectPr>");

    // IMPORTANT: Elements must appear in the order specified by the OOXML schema!
    // The order is: footnotePr, endnotePr, type, pgSz, pgMar, paperSrc, pgBorders,
    // lnNumType, pgNumType, cols, formProt, vAlign, noEndnote, titlePg, textDirection,
    // bidi, rtlGutter, docGrid, printerSettings, headerReference, footerReference, sectPrChange

    // Footnote properties (preserve from original template)
    xml.push_str(
        r#"<w:footnotePr><w:numFmt w:val="decimal"/><w:numRestart w:val="eachSect"/></w:footnotePr>"#,
    );

    // Add header references if header exists (must come after footnotePr, before type)
    if has_header {
        if has_first_header {
            // First page header (header2.xml)
            xml.push_str(r#"<w:headerReference w:type="first" r:id="rIdHeader2"/>"#);
        }
        // Default header (header1.xml)
        xml.push_str(r#"<w:headerReference w:type="default" r:id="rIdHeader1"/>"#);
    }

    // Add footer references if footer exists
    if has_footer {
        if has_first_footer {
            // First page footer (footer2.xml)
            xml.push_str(r#"<w:footerReference w:type="first" r:id="rIdFooter2"/>"#);
        }
        // Default footer (footer1.xml)
        xml.push_str(r#"<w:footerReference w:type="default" r:id="rIdFooter1"/>"#);
    }

    // Section type (continuous, nextPage, etc.)
    xml.push_str(r#"<w:type w:val="nextPage"/>"#);

    // Page size
    let (width, height) = get_var(vars, "page-size")
        .and_then(|size| page_size_to_twips(&size))
        .unwrap_or((11906, 16838)); // Default to A4

    xml.push_str(&format!(r#"<w:pgSz w:w="{width}" w:h="{height}"/>"#));

    // Page margins
    // Get margin values in twips (already converted by theme processor)
    let margin_top = get_twips_u32(vars, "page-margin-top").unwrap_or(1440); // Default: 1 inch
    let margin_right = get_twips_u32(vars, "page-margin-right").unwrap_or(1440);
    let margin_bottom = get_twips_u32(vars, "page-margin-bottom").unwrap_or(1440);
    let margin_left = get_twips_u32(vars, "page-margin-left").unwrap_or(1440);

    // Header and footer padding (distance from page edge to header/footer content)
    // These map to w:header and w:footer attributes in w:pgMar
    let header_padding = get_twips_u32(vars, "page-padding-top").unwrap_or(720); // Default: 0.5 inch
    let footer_padding = get_twips_u32(vars, "page-padding-bottom").unwrap_or(720);

    xml.push_str(&format!(
        r#"<w:pgMar w:left="{margin_left}" w:right="{margin_right}" w:gutter="0" w:header="{header_padding}" w:top="{margin_top}" w:footer="{footer_padding}" w:bottom="{margin_bottom}"/>"#
    ));

    // Page numbering type
    xml.push_str(r#"<w:pgNumType w:fmt="decimal"/>"#);

    // Form protection
    xml.push_str(r#"<w:formProt w:val="false"/>"#);

    // Different first page flag (comes after formProt)
    if has_first_header || has_first_footer {
        xml.push_str(r#"<w:titlePg/>"#);
    }

    // Text direction
    xml.push_str(r#"<w:textDirection w:val="lrTb"/>"#);

    // Document grid (preserve from original template)
    xml.push_str(r#"<w:docGrid w:type="default" w:linePitch="100" w:charSpace="0"/>"#);

    xml.push_str("</w:sectPr>");
    xml
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_page_size_conversion() {
        // A4: 210mm × 297mm = 11906 × 16838 twips
        assert_eq!(page_size_to_twips("a4"), Some((11906, 16838)));
        assert_eq!(page_size_to_twips("A4"), Some((11906, 16838)));

        // Letter: 8.5in × 11in = 12240 × 15840 twips
        assert_eq!(page_size_to_twips("letter"), Some((12240, 15840)));
        assert_eq!(page_size_to_twips("us-letter"), Some((12240, 15840)));

        // Unknown size
        assert_eq!(page_size_to_twips("unknown"), None);
    }

    #[test]
    fn test_section_properties_basic() {
        let mut vars = BTreeMap::new();
        vars.insert("page-size".to_string(), json!("a4"));
        vars.insert("page-margin-top".to_string(), json!(1440.0));
        vars.insert("page-margin-right".to_string(), json!(1440.0));
        vars.insert("page-margin-bottom".to_string(), json!(1440.0));
        vars.insert("page-margin-left".to_string(), json!(1440.0));

        let xml = build_section_properties(&vars, false, false, false, false);

        // Verify structure
        assert!(xml.contains("<w:sectPr>"));
        assert!(xml.contains("</w:sectPr>"));

        // Verify required elements are present
        assert!(xml.contains("<w:footnotePr>"));
        assert!(xml.contains(r#"<w:type w:val="nextPage"/>"#));
        assert!(xml.contains(r#"<w:pgSz w:w="11906" w:h="16838"/>"#));
        assert!(xml.contains(r#"w:top="1440""#));
        assert!(xml.contains(r#"w:bottom="1440""#));
        assert!(xml.contains(r#"<w:pgNumType w:fmt="decimal"/>"#));
        assert!(xml.contains(r#"<w:formProt w:val="false"/>"#));
        assert!(xml.contains(r#"<w:textDirection w:val="lrTb"/>"#));
        assert!(xml.contains("<w:docGrid"));

        // Verify no headers/footers when not requested
        assert!(!xml.contains("w:headerReference"));
        assert!(!xml.contains("w:footerReference"));
        assert!(!xml.contains("<w:titlePg"));
    }

    #[test]
    fn test_section_properties_with_header_footer() {
        let vars = BTreeMap::new();
        let xml = build_section_properties(&vars, true, true, false, false);

        assert!(xml.contains(r#"<w:headerReference w:type="default" r:id="rIdHeader1"/>"#));
        assert!(xml.contains(r#"<w:footerReference w:type="default" r:id="rIdFooter1"/>"#));
        assert!(!xml.contains(r#"w:type="first""#));
    }

    #[test]
    fn test_section_properties_with_first_page() {
        let vars = BTreeMap::new();
        let xml = build_section_properties(&vars, true, true, true, true);

        // Verify header/footer references come before type element
        let type_pos = xml.find(r#"<w:type"#).expect("type element should exist");
        let header_pos = xml
            .find("headerReference")
            .expect("header reference should exist");
        assert!(
            header_pos < type_pos,
            "headerReference should come before type"
        );

        // Verify titlePg comes after formProt
        let form_prot_pos = xml.find("formProt").expect("formProt should exist");
        let title_pg_pos = xml.find("titlePg").expect("titlePg should exist");
        assert!(
            title_pg_pos > form_prot_pos,
            "titlePg should come after formProt"
        );

        // Verify all expected elements
        assert!(xml.contains("<w:titlePg/>"));
        assert!(xml.contains(r#"<w:headerReference w:type="first" r:id="rIdHeader2"/>"#));
        assert!(xml.contains(r#"<w:headerReference w:type="default" r:id="rIdHeader1"/>"#));
        assert!(xml.contains(r#"<w:footerReference w:type="first" r:id="rIdFooter2"/>"#));
        assert!(xml.contains(r#"<w:footerReference w:type="default" r:id="rIdFooter1"/>"#));
    }

    #[test]
    fn test_section_properties_element_order() {
        // This test verifies the CRITICAL element ordering required by OOXML spec
        let vars = BTreeMap::new();
        let xml = build_section_properties(&vars, true, true, true, true);

        // Element positions (all must be in ascending order)
        let footnote_pos = xml.find("footnotePr").expect("footnotePr should exist");
        let header_pos = xml
            .find("headerReference")
            .expect("headerReference should exist");
        let footer_pos = xml
            .find("footerReference")
            .expect("footerReference should exist");
        let type_pos = xml.find("<w:type").expect("type should exist");
        let pg_sz_pos = xml.find("pgSz").expect("pgSz should exist");
        let pg_mar_pos = xml.find("pgMar").expect("pgMar should exist");
        let pg_num_pos = xml.find("pgNumType").expect("pgNumType should exist");
        let form_prot_pos = xml.find("formProt").expect("formProt should exist");
        let title_pg_pos = xml.find("titlePg").expect("titlePg should exist");
        let text_dir_pos = xml
            .find("textDirection")
            .expect("textDirection should exist");
        let doc_grid_pos = xml.find("docGrid").expect("docGrid should exist");

        // Verify order (based on OOXML spec)
        assert!(
            footnote_pos < header_pos,
            "footnotePr before headerReference"
        );
        assert!(
            header_pos < footer_pos,
            "headerReference before footerReference"
        );
        assert!(footer_pos < type_pos, "footerReference before type");
        assert!(type_pos < pg_sz_pos, "type before pgSz");
        assert!(pg_sz_pos < pg_mar_pos, "pgSz before pgMar");
        assert!(pg_mar_pos < pg_num_pos, "pgMar before pgNumType");
        assert!(pg_num_pos < form_prot_pos, "pgNumType before formProt");
        assert!(form_prot_pos < title_pg_pos, "formProt before titlePg");
        assert!(title_pg_pos < text_dir_pos, "titlePg before textDirection");
        assert!(text_dir_pos < doc_grid_pos, "textDirection before docGrid");
    }
}
