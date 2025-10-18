use image::GenericImageView;
use serde_json::Value;
use std::collections::BTreeMap;

use crate::encode_utils::{
    build_paragraph_border, build_tab_stops, extract_url, get_color_hex, get_font_size_half_points,
    get_var, resolve_border_tokens,
};

/// Maximum height for images in headers/footers (0.5 inches = 720 twips)
const MAX_HEADER_FOOTER_IMAGE_HEIGHT_TWIPS: u32 = 720;

/// Check if bytes represent an SVG file
///
/// Detects SVG by looking for XML declaration or <svg tag at the start
fn is_svg(bytes: &[u8]) -> bool {
    if bytes.len() < 5 {
        return false;
    }

    // Check for common SVG starting patterns
    let start = &bytes[..bytes.len().min(200)];
    let as_str = String::from_utf8_lossy(start).to_lowercase();

    as_str.starts_with("<?xml") || as_str.starts_with("<svg") || as_str.contains("<svg ")
}

/// Parse dimensions from SVG content
///
/// Extracts width and height from SVG attributes or viewBox.
/// Returns dimensions in pixels, or None if parsing fails.
fn parse_svg_dimensions(svg_bytes: &[u8]) -> Option<(u32, u32)> {
    // Parse the SVG XML
    let text = String::from_utf8_lossy(svg_bytes);
    let doc = roxmltree::Document::parse(&text).ok()?;

    // Find the root <svg> element
    let svg = doc.root_element();

    // Try to get width and height from attributes
    let width = svg.attribute("width").and_then(parse_svg_length);
    let height = svg.attribute("height").and_then(parse_svg_length);

    if let (Some(w), Some(h)) = (width, height) {
        return Some((w, h));
    }

    // Fall back to viewBox if width/height not present
    if let Some(viewbox) = svg.attribute("viewBox") {
        let parts: Vec<&str> = viewbox.split_whitespace().collect();
        if parts.len() == 4 {
            let w = parts[2].parse::<f64>().ok()?;
            let h = parts[3].parse::<f64>().ok()?;
            return Some((w as u32, h as u32));
        }
    }

    // Default size if no dimensions found
    Some((300, 150))
}

/// Parse SVG length attribute to pixels
///
/// Handles various SVG units (px, pt, in, cm, mm, etc.)
/// Returns pixel value assuming 96 DPI
fn parse_svg_length(value: &str) -> Option<u32> {
    let value = value.trim();

    // Extract number and unit
    let (num_str, unit) = if let Some(stripped) = value.strip_suffix("px") {
        (stripped, "px")
    } else if let Some(stripped) = value.strip_suffix("pt") {
        (stripped, "pt")
    } else if let Some(stripped) = value.strip_suffix("in") {
        (stripped, "in")
    } else if let Some(stripped) = value.strip_suffix("cm") {
        (stripped, "cm")
    } else if let Some(stripped) = value.strip_suffix("mm") {
        (stripped, "mm")
    } else if value.ends_with('%') {
        return None; // Percentage requires context
    } else {
        (value, "px") // Default to pixels
    };

    let num = num_str.parse::<f64>().ok()?;

    // Convert to pixels at 96 DPI
    let pixels = match unit {
        "px" => num,
        "pt" => num * 96.0 / 72.0,
        "in" => num * 96.0,
        "cm" => num * 96.0 / 2.54,
        "mm" => num * 96.0 / 25.4,
        _ => num,
    };

    Some(pixels as u32)
}

/// Fetch an image from a URL and prepare it for embedding in DOCX
///
/// Downloads the image, detects format and dimensions, and scales if needed
/// to fit within the maximum header/footer height while maintaining aspect ratio.
///
/// # Arguments
/// * `url` - The URL to fetch the image from
/// * `max_height_twips` - Maximum height in twips (typically 720 for headers/footers)
///
/// # Returns
/// Optional tuple of (image_bytes, file_extension, width_twips, height_twips)
/// Returns None if fetch or image processing fails (with warning logged)
async fn fetch_and_prepare_image(
    url: &str,
    max_height_twips: u32,
) -> Option<(Vec<u8>, String, u32, u32)> {
    // Fetch the image
    let response = match reqwest::get(url).await {
        Ok(resp) => resp,
        Err(e) => {
            tracing::warn!("Failed to fetch image from {}: {}", url, e);
            return None;
        }
    };

    if !response.status().is_success() {
        tracing::warn!(
            "Failed to fetch image from {}: HTTP {}",
            url,
            response.status()
        );
        return None;
    }

    let bytes = match response.bytes().await {
        Ok(b) => b.to_vec(),
        Err(e) => {
            tracing::warn!("Failed to read image bytes from {}: {}", url, e);
            return None;
        }
    };

    // Check if this is an SVG file
    if is_svg(&bytes) {
        // Parse SVG dimensions
        let (width_px, height_px) = match parse_svg_dimensions(&bytes) {
            Some(dims) => dims,
            None => {
                tracing::warn!("Failed to parse SVG dimensions from {}", url);
                return None;
            }
        };

        // Convert pixels to twips (assuming 96 DPI: 1px = 15 twips)
        const PX_TO_TWIPS: f64 = 15.0;
        let width_twips = (width_px as f64 * PX_TO_TWIPS) as u32;
        let mut height_twips = (height_px as f64 * PX_TO_TWIPS) as u32;

        // Scale down if height exceeds maximum, maintaining aspect ratio
        let final_width_twips = if height_twips > max_height_twips {
            let scale = max_height_twips as f64 / height_twips as f64;
            height_twips = max_height_twips;
            (width_twips as f64 * scale) as u32
        } else {
            width_twips
        };

        return Some((bytes, "svg".to_string(), final_width_twips, height_twips));
    }

    // Handle raster images using the image crate
    let img = match image::load_from_memory(&bytes) {
        Ok(img) => img,
        Err(e) => {
            tracing::warn!("Failed to decode image from {}: {}", url, e);
            return None;
        }
    };

    // Detect format from the loaded image
    let format = img.color();
    let extension = match format {
        image::ColorType::Rgba8
        | image::ColorType::Rgb8
        | image::ColorType::La8
        | image::ColorType::L8 => "png",
        _ => "png", // Default to PNG for other formats
    }
    .to_string();

    // Get dimensions in pixels
    let (width_px, height_px) = img.dimensions();

    // Convert pixels to twips (assuming 96 DPI: 1px = 15 twips)
    const PX_TO_TWIPS: f64 = 15.0;
    let width_twips = (width_px as f64 * PX_TO_TWIPS) as u32;
    let mut height_twips = (height_px as f64 * PX_TO_TWIPS) as u32;

    // Scale down if height exceeds maximum, maintaining aspect ratio
    let final_width_twips = if height_twips > max_height_twips {
        let scale = max_height_twips as f64 / height_twips as f64;
        height_twips = max_height_twips;
        (width_twips as f64 * scale) as u32
    } else {
        width_twips
    };

    Some((bytes, extension, final_width_twips, height_twips))
}

/// Build a run containing an embedded image
///
/// Fetches the image, adds it to media_files, and generates DOCX drawing XML.
/// Returns None if image fetch fails (with warning logged).
///
/// # Arguments
/// * `url` - The URL to fetch the image from
/// * `media_files` - Collection to add the fetched image bytes
/// * `base_index` - Starting index for image numbering (to avoid conflicts with existing media)
/// * `max_height_twips` - Maximum height for the image
async fn build_image_run(
    url: &str,
    media_files: &mut Vec<(String, Vec<u8>)>,
    base_index: usize,
    max_height_twips: u32,
) -> Option<String> {
    // Fetch and prepare the image
    let (bytes, extension, width_twips, height_twips) =
        fetch_and_prepare_image(url, max_height_twips).await?;

    // Add to media files and get the relationship ID (1-based index + base offset)
    let rel_id = media_files.len() + 1;
    let image_number = base_index + rel_id;
    let filename = format!("image{}.{}", image_number, extension);
    media_files.push((filename.clone(), bytes));

    // Convert twips to EMUs (English Metric Units: 1 twip = 635 EMUs)
    let width_emus = width_twips as u64 * 635;
    let height_emus = height_twips as u64 * 635;

    // Generate unique IDs for the drawing elements
    let doc_pr_id = rel_id;
    let c_nv_pr_id = rel_id;

    // Build the drawing XML
    Some(format!(
        r#"<w:r><w:drawing><wp:inline distT="0" distB="0" distL="0" distR="0"><wp:extent cx="{width_emus}" cy="{height_emus}"/><wp:effectExtent l="0" t="0" r="0" b="0"/><wp:docPr id="{doc_pr_id}" name="Picture {doc_pr_id}"/><wp:cNvGraphicFramePr><a:graphicFrameLocks noChangeAspect="1"/></wp:cNvGraphicFramePr><a:graphic><a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/picture"><pic:pic><pic:nvPicPr><pic:cNvPr id="{c_nv_pr_id}" name="{filename}"/><pic:cNvPicPr/></pic:nvPicPr><pic:blipFill><a:blip r:embed="rId{rel_id}"/><a:stretch><a:fillRect/></a:stretch></pic:blipFill><pic:spPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="{width_emus}" cy="{height_emus}"/></a:xfrm><a:prstGeom prst="rect"><a:avLst/></a:prstGeom></pic:spPr></pic:pic></a:graphicData></a:graphic></wp:inline></w:drawing></w:r>"#
    ))
}

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
/// - url() values → embedded images
///
/// # Arguments
/// * `vars` - Pre-computed theme variables
/// * `left_content` - Content for left-aligned position (token name)
/// * `center_content` - Content for center-aligned position (token name)
/// * `right_content` - Content for right-aligned position (token name)
/// * `page_width` - Page content width in twips for tab stop positioning
/// * `media_files` - Collection to track embedded media files
/// * `base_index` - Starting index for image numbering (to avoid conflicts with existing media)
///
/// # Returns
/// Complete header XML file content, or None if no content defined
pub(crate) async fn build_header_xml(
    vars: &BTreeMap<String, Value>,
    left_content: &str,
    center_content: &str,
    right_content: &str,
    page_width: u32,
    media_files: &mut Vec<(String, Vec<u8>)>,
    base_index: usize,
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
<w:hdr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:pic="http://schemas.openxmlformats.org/drawingml/2006/picture">
<w:p><w:pPr>"#,
    );

    // Add tab stops for left/center/right positioning
    xml.push_str(&build_tab_stops(page_width));

    // Add border if defined
    if let Some((width, color, style)) = resolve_border_tokens(vars, "page-top-border") {
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
        if let Some(url) = extract_url(&left) {
            if let Some(image_xml) = build_image_run(
                &url,
                media_files,
                base_index,
                MAX_HEADER_FOOTER_IMAGE_HEIGHT_TWIPS,
            )
            .await
            {
                xml.push_str(&image_xml);
            }
        } else {
            xml.push_str(&format!(
                r#"<w:r><w:rPr>{char_props}</w:rPr><w:t xml:space="preserve">{left}</w:t></w:r>"#
            ));
        }
    }

    // Tab to center
    if !center.is_empty() {
        xml.push_str(&format!(
            r#"<w:r><w:rPr>{char_props}</w:rPr><w:tab/></w:r>"#
        ));
        if let Some(url) = extract_url(&center) {
            if let Some(image_xml) = build_image_run(
                &url,
                media_files,
                base_index,
                MAX_HEADER_FOOTER_IMAGE_HEIGHT_TWIPS,
            )
            .await
            {
                xml.push_str(&image_xml);
            }
        } else {
            xml.push_str(&format!(
                r#"<w:r><w:rPr>{char_props}</w:rPr><w:t xml:space="preserve">{center}</w:t></w:r>"#
            ));
        }
    }

    // Tab to right
    if !right.is_empty() {
        xml.push_str(&format!(
            r#"<w:r><w:rPr>{char_props}</w:rPr><w:tab/></w:r>"#
        ));
        if let Some(url) = extract_url(&right) {
            if let Some(image_xml) = build_image_run(
                &url,
                media_files,
                base_index,
                MAX_HEADER_FOOTER_IMAGE_HEIGHT_TWIPS,
            )
            .await
            {
                xml.push_str(&image_xml);
            }
        } else {
            xml.push_str(&format!(
                r#"<w:r><w:rPr>{char_props}</w:rPr><w:t xml:space="preserve">{right}</w:t></w:r>"#
            ));
        }
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
/// - url() values → embedded images
///
/// # Arguments
/// * `vars` - Pre-computed theme variables
/// * `left_content` - Content for left-aligned position (token name)
/// * `center_content` - Content for center-aligned position (token name)
/// * `right_content` - Content for right-aligned position (token name)
/// * `page_width` - Page content width in twips for tab stop positioning
/// * `media_files` - Collection to track embedded media files
/// * `base_index` - Starting index for image numbering (to avoid conflicts with existing media)
///
/// # Returns
/// Complete footer XML file content, or None if no content defined
pub(crate) async fn build_footer_xml(
    vars: &BTreeMap<String, Value>,
    left_content: &str,
    center_content: &str,
    right_content: &str,
    page_width: u32,
    media_files: &mut Vec<(String, Vec<u8>)>,
    base_index: usize,
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
<w:ftr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:pic="http://schemas.openxmlformats.org/drawingml/2006/picture">
<w:p><w:pPr>"#,
    );

    // Add tab stops for left/center/right positioning
    xml.push_str(&build_tab_stops(page_width));

    // Add border if defined
    if let Some((width, color, style)) = resolve_border_tokens(vars, "page-bottom-border") {
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
        if let Some(url) = extract_url(&left) {
            if let Some(image_xml) = build_image_run(
                &url,
                media_files,
                base_index,
                MAX_HEADER_FOOTER_IMAGE_HEIGHT_TWIPS,
            )
            .await
            {
                xml.push_str(&image_xml);
            }
        } else {
            xml.push_str(&format!(
                r#"<w:r><w:rPr>{char_props}</w:rPr><w:t xml:space="preserve">{left}</w:t></w:r>"#
            ));
        }
    }

    // Tab to center
    if !center.is_empty() {
        xml.push_str(&format!(
            r#"<w:r><w:rPr>{char_props}</w:rPr><w:tab/></w:r>"#
        ));
        if let Some(url) = extract_url(&center) {
            if let Some(image_xml) = build_image_run(
                &url,
                media_files,
                base_index,
                MAX_HEADER_FOOTER_IMAGE_HEIGHT_TWIPS,
            )
            .await
            {
                xml.push_str(&image_xml);
            }
        } else {
            xml.push_str(&format!(
                r#"<w:r><w:rPr>{char_props}</w:rPr><w:t xml:space="preserve">{center}</w:t></w:r>"#
            ));
        }
    }

    // Tab to right
    if !right.is_empty() {
        xml.push_str(&format!(
            r#"<w:r><w:rPr>{char_props}</w:rPr><w:tab/></w:r>"#
        ));
        if let Some(url) = extract_url(&right) {
            if let Some(image_xml) = build_image_run(
                &url,
                media_files,
                base_index,
                MAX_HEADER_FOOTER_IMAGE_HEIGHT_TWIPS,
            )
            .await
            {
                xml.push_str(&image_xml);
            }
        } else {
            xml.push_str(&format!(
                r#"<w:r><w:rPr>{char_props}</w:rPr><w:t xml:space="preserve">{right}</w:t></w:r>"#
            ));
        }
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

        let border = resolve_border_tokens(&vars, "page-top-border");

        assert!(border.is_some());
        if let Some((width, color, style)) = border {
            assert_eq!(width, "60");
            assert_eq!(color, "FF0000");
            assert_eq!(style, "solid");
        }
    }

    #[test]
    fn test_resolve_border_tokens_pre_resolved() {
        let mut vars = BTreeMap::new();
        // Simulate CSS variables already resolved via computed_variables_with_overrides
        // If page-top-border-* wasn't explicitly set, the theme CSS would have already
        // resolved it to the value from page-horizontal-border-* via var()
        vars.insert("page-top-border-width".to_string(), json!(40.0));
        vars.insert("page-top-border-color".to_string(), json!("#00FF00"));
        vars.insert("page-top-border-style".to_string(), json!("dashed"));

        let border = resolve_border_tokens(&vars, "page-top-border");

        assert!(border.is_some());
        if let Some((width, color, style)) = border {
            assert_eq!(width, "40");
            assert_eq!(color, "00FF00");
            assert_eq!(style, "dashed");
        }
    }

    #[tokio::test]
    async fn test_build_header_xml_with_content() {
        let mut vars = BTreeMap::new();
        vars.insert("page-top-left-content".to_string(), json!("Left"));
        vars.insert("page-top-center-content".to_string(), json!("Center"));
        vars.insert("page-top-right-content".to_string(), json!("Right"));
        vars.insert("page-margin-font-family".to_string(), json!("Arial"));
        vars.insert("page-margin-font-size".to_string(), json!(160.0)); // 8pt in twips
        vars.insert("page-margin-color".to_string(), json!("#333333"));

        let mut media_files = Vec::new();
        let xml = build_header_xml(
            &vars,
            "page-top-left-content",
            "page-top-center-content",
            "page-top-right-content",
            9000,
            &mut media_files,
            0,
        )
        .await;

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

    #[tokio::test]
    async fn test_build_header_xml_empty() {
        let vars = BTreeMap::new();

        let mut media_files = Vec::new();
        let xml = build_header_xml(
            &vars,
            "page-top-left-content",
            "page-top-center-content",
            "page-top-right-content",
            9000,
            &mut media_files,
            0,
        )
        .await;

        assert!(xml.is_none());
    }

    #[tokio::test]
    async fn test_build_footer_xml_with_border() {
        let mut vars = BTreeMap::new();
        vars.insert("page-bottom-center-content".to_string(), json!("Page"));
        vars.insert("page-bottom-border-width".to_string(), json!(40.0));
        vars.insert("page-bottom-border-color".to_string(), json!("#000000"));
        vars.insert("page-bottom-border-style".to_string(), json!("single"));

        let mut media_files = Vec::new();
        let xml = build_footer_xml(
            &vars,
            "page-bottom-left-content",
            "page-bottom-center-content",
            "page-bottom-right-content",
            9000,
            &mut media_files,
            0,
        )
        .await;

        assert!(xml.is_some());
        if let Some(xml) = xml {
            assert!(xml.contains("<w:ftr"));
            assert!(xml.contains("Page"));
            assert!(xml.contains("<w:pBdr><w:top"));
            assert!(xml.contains("000000"));
        }
    }

    #[test]
    fn test_is_svg_with_xml_declaration() {
        let svg = b"<?xml version=\"1.0\"?><svg xmlns=\"http://www.w3.org/2000/svg\"></svg>";
        assert!(is_svg(svg));
    }

    #[test]
    fn test_is_svg_with_svg_tag() {
        let svg = b"<svg width=\"100\" height=\"100\"></svg>";
        assert!(is_svg(svg));
    }

    #[test]
    fn test_is_svg_not_svg() {
        let png = b"\x89PNG\r\n\x1a\n";
        assert!(!is_svg(png));
    }

    #[test]
    fn test_parse_svg_dimensions_with_attributes() {
        let svg = b"<svg width=\"200\" height=\"100\"></svg>";
        let dims = parse_svg_dimensions(svg);
        assert_eq!(dims, Some((200, 100)));
    }

    #[test]
    fn test_parse_svg_dimensions_with_units() {
        let svg = b"<svg width=\"2in\" height=\"1in\"></svg>";
        let dims = parse_svg_dimensions(svg);
        assert_eq!(dims, Some((192, 96))); // 2in * 96dpi, 1in * 96dpi
    }

    #[test]
    fn test_parse_svg_dimensions_with_viewbox() {
        let svg = b"<svg viewBox=\"0 0 400 300\"></svg>";
        let dims = parse_svg_dimensions(svg);
        assert_eq!(dims, Some((400, 300)));
    }

    #[test]
    fn test_parse_svg_dimensions_default() {
        let svg = b"<svg></svg>";
        let dims = parse_svg_dimensions(svg);
        assert_eq!(dims, Some((300, 150))); // Default size
    }

    #[test]
    fn test_parse_svg_length_pixels() {
        assert_eq!(parse_svg_length("100"), Some(100));
        assert_eq!(parse_svg_length("100px"), Some(100));
    }

    #[test]
    fn test_parse_svg_length_inches() {
        assert_eq!(parse_svg_length("1in"), Some(96));
        assert_eq!(parse_svg_length("2in"), Some(192));
    }

    #[test]
    fn test_parse_svg_length_points() {
        assert_eq!(parse_svg_length("72pt"), Some(96)); // 72pt = 1in = 96px
    }

    #[test]
    fn test_parse_svg_length_cm() {
        assert_eq!(parse_svg_length("2.54cm"), Some(96)); // 2.54cm = 1in = 96px
    }
}
