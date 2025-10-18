use std::{
    collections::BTreeMap,
    fs::File,
    io::{Read, Write},
    path::Path,
};

use regex::Regex;
use stencila_themes::Theme;
use uuid::Uuid;
use zip::{ZipArchive, ZipWriter, write::SimpleFileOptions};

use stencila_codec::eyre::{Context, OptionExt, Result, eyre};
use stencila_codec_utils::move_file;

use crate::encode_fonts::{embed_fonts, resolve_fonts};
use crate::encode_headers_footers::{build_footer_xml, build_header_xml};
use crate::encode_page_layout::build_section_properties;
use crate::encode_theme::theme_to_styles;
use crate::encode_utils::escape_xml;

/// Find the highest image number in existing media files
///
/// Scans the DOCX parts for files matching `word/media/imageN.ext` pattern
/// and returns the highest N found, or 0 if no matching files exist.
/// This ensures new header/footer images don't overwrite existing media.
fn find_max_media_number(parts: &BTreeMap<String, Vec<u8>>) -> usize {
    parts
        .keys()
        .filter_map(|path| {
            // Match "word/media/imageN.ext" pattern
            if let Some(rest) = path.strip_prefix("word/media/image") {
                // Extract the number before the extension
                rest.split('.').next().and_then(|n| n.parse::<usize>().ok())
            } else {
                None
            }
        })
        .max()
        .unwrap_or(0)
}

/// Encode custom data, properties & theme into a DOCX
pub async fn apply(
    path: &Path,
    data: Vec<(String, String)>,
    properties: Vec<(String, String)>,
    theme: Option<Theme>,
    document_variables: Option<BTreeMap<String, String>>,
) -> Result<()> {
    if data.is_empty() && properties.is_empty() && theme.is_none() {
        return Ok(());
    }

    // Read the DOCX (ZIP) into a BTreeMap for easy lookup and replacement.
    let mut docx =
        File::open(path).wrap_err_with(|| eyre!("unable to open: {}", path.display()))?;
    let mut zip = ZipArchive::new(&mut docx)
        .wrap_err_with(|| eyre!("DOCX is not a valid zip: {}", path.display()))?;
    let mut parts: BTreeMap<String, Vec<u8>> = BTreeMap::new();

    // Read all existing files
    for index in 0..zip.len() {
        let mut file = zip.by_index(index)?;
        let mut buffer = Vec::with_capacity(file.size() as usize);
        file.read_to_end(&mut buffer)?;
        parts.insert(file.name().to_owned(), buffer);
    }

    // Fetch or create [Content_Types].xml.
    const CONTENT_TYPES: &str = "[Content_Types].xml";
    let mut content_types_xml = if let Some(bytes) = parts.get(CONTENT_TYPES) {
        String::from_utf8(bytes.clone())?
    } else {
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"></Types>"#
            .to_string()
    };

    // Fetch or create docProps/custom.xml.
    const CUSTOM_PROPS: &str = "docProps/custom.xml";
    let mut custom_props_xml = if let Some(bytes) = parts.get(CUSTOM_PROPS) {
        String::from_utf8(bytes.clone())?
    } else {
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Properties xmlns="http://schemas.openxmlformats.org/officeDocument/2006/custom-properties" xmlns:vt="http://schemas.openxmlformats.org/officeDocument/2006/docPropsVTypes"></Properties>"#
            .to_string()
    };

    // Fetch or create word/_rels/document.xml.rels.
    const DOCUMENT_RELS: &str = "word/_rels/document.xml.rels";
    let mut rels_xml = if let Some(bytes) = parts.get(DOCUMENT_RELS) {
        String::from_utf8(bytes.clone())?
    } else {
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"></Relationships>"#
            .to_string()
    };

    // Insert each data embedding as /customXml/*.xml plus its property part.
    for (index, (name, payload)) in data.into_iter().enumerate() {
        // It is necessary for the starting id to be "1", at least for round-trip
        // preservation after editing with LibreOffice
        let id = index + 1;
        let data_part = format!("customXml/item{id}.xml");
        let props_part = format!("customXml/itemProps{id}.xml");
        let rel_part = format!("customXml/_rels/item{id}.xml.rels");

        let data_xml = format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<data name="{name}"><![CDATA[{payload}]]></data>"#
        );
        parts.insert(data_part.clone(), data_xml.into_bytes());

        let guid = Uuid::new_v4().to_string();
        let props_xml = format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<ds:datastoreItem ds:itemID="{{{guid}}}" xmlns:ds="http://schemas.openxmlformats.org/officeDocument/2006/customXml"/>"#
        );
        parts.insert(props_part.clone(), props_xml.into_bytes());

        parts.insert(rel_part, r#"<?xml version='1.0' encoding='UTF-8' standalone='yes'?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/customXmlProps" Target="itemProps1.xml"/>
</Relationships>
"#.into());

        content_types_xml = insert_override(
            &content_types_xml,
            &format!("/{data_part}"),
            "application/xml",
        )?;
        content_types_xml = insert_override(
            &content_types_xml,
            &format!("/{props_part}"),
            "application/vnd.openxmlformats-officedocument.customXmlProperties+xml",
        )?;

        rels_xml = insert_relationship(&rels_xml, &format!("../{data_part}"))?;
    }

    // Append or extend custom document properties.
    custom_props_xml = add_custom_props(&custom_props_xml, properties)?;

    // Generate word/styles.xml and embed fonts if a theme is specified
    const STYLES: &str = "word/styles.xml";
    const DOCUMENT: &str = "word/document.xml";
    if let Some(theme) = theme.as_ref() {
        // Get computed theme variables in twips
        let theme_variables = theme.computed_variables_with_overrides(
            stencila_themes::LengthConversion::Twips,
            document_variables.unwrap_or_default(),
        );

        // Resolve fonts from CSS stacks and get resolved font bytes
        let (resolved_fonts, resolved_variables) = resolve_fonts(&theme_variables).await?;

        // Generate styles.xml using the resolved variables (with font names instead of CSS stacks)
        let styles_xml = theme_to_styles(&resolved_variables);
        parts.insert(STYLES.to_string(), styles_xml.into_bytes());

        // Embed fonts if any were resolved
        if !resolved_fonts.is_empty() {
            embed_fonts(
                &mut parts,
                &resolved_fonts,
                &mut content_types_xml,
                &mut rels_xml,
            )?;
        }

        // Calculate page width for header/footer tab stops
        // Try to get from theme, otherwise default to A4 width minus margins
        let page_width = resolved_variables
            .get("page-content-width")
            .and_then(|v| v.as_f64())
            .unwrap_or(9026.0) as u32; // A4 (11906 twips) - 2*1440 margin = 9026 twips

        // Find highest existing media number to avoid filename conflicts
        let media_start_index = find_max_media_number(&parts);

        // Create collection for media files embedded in headers/footers
        let mut media_files: Vec<(String, Vec<u8>)> = Vec::new();

        // Generate headers if any header content is defined
        let header1 = build_header_xml(
            &resolved_variables,
            "page-top-left-content",
            "page-top-center-content",
            "page-top-right-content",
            page_width,
            &mut media_files,
            media_start_index,
        )
        .await;
        let header2 = build_header_xml(
            &resolved_variables,
            "page1-top-left-content",
            "page1-top-center-content",
            "page1-top-right-content",
            page_width,
            &mut media_files,
            media_start_index,
        )
        .await;

        // Generate footers if any footer content is defined
        let footer1 = build_footer_xml(
            &resolved_variables,
            "page-bottom-left-content",
            "page-bottom-center-content",
            "page-bottom-right-content",
            page_width,
            &mut media_files,
            media_start_index,
        )
        .await;
        let footer2 = build_footer_xml(
            &resolved_variables,
            "page1-bottom-left-content",
            "page1-bottom-center-content",
            "page1-bottom-right-content",
            page_width,
            &mut media_files,
            media_start_index,
        )
        .await;

        // Check which headers/footers exist before consuming them
        let has_header = header1.is_some();
        let has_footer = footer1.is_some();
        let has_first_header = header2.is_some();
        let has_first_footer = footer2.is_some();

        // Insert header XML files if they exist
        if let Some(xml) = header1 {
            parts.insert("word/header1.xml".to_string(), xml.into_bytes());
            content_types_xml = insert_override(
                &content_types_xml,
                "/word/header1.xml",
                "application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml",
            )?;
            rels_xml = insert_relationship_with_id(
                &rels_xml,
                "rIdHeader1",
                "http://schemas.openxmlformats.org/officeDocument/2006/relationships/header",
                "header1.xml",
            )?;
        }

        if let Some(xml) = header2 {
            parts.insert("word/header2.xml".to_string(), xml.into_bytes());
            content_types_xml = insert_override(
                &content_types_xml,
                "/word/header2.xml",
                "application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml",
            )?;
            rels_xml = insert_relationship_with_id(
                &rels_xml,
                "rIdHeader2",
                "http://schemas.openxmlformats.org/officeDocument/2006/relationships/header",
                "header2.xml",
            )?;
        }

        // Insert footer XML files if they exist
        if let Some(xml) = footer1 {
            parts.insert("word/footer1.xml".to_string(), xml.into_bytes());
            content_types_xml = insert_override(
                &content_types_xml,
                "/word/footer1.xml",
                "application/vnd.openxmlformats-officedocument.wordprocessingml.footer+xml",
            )?;
            rels_xml = insert_relationship_with_id(
                &rels_xml,
                "rIdFooter1",
                "http://schemas.openxmlformats.org/officeDocument/2006/relationships/footer",
                "footer1.xml",
            )?;
        }

        if let Some(xml) = footer2 {
            parts.insert("word/footer2.xml".to_string(), xml.into_bytes());
            content_types_xml = insert_override(
                &content_types_xml,
                "/word/footer2.xml",
                "application/vnd.openxmlformats-officedocument.wordprocessingml.footer+xml",
            )?;
            rels_xml = insert_relationship_with_id(
                &rels_xml,
                "rIdFooter2",
                "http://schemas.openxmlformats.org/officeDocument/2006/relationships/footer",
                "footer2.xml",
            )?;
        }

        // Handle media files embedded in headers/footers
        if !media_files.is_empty() {
            // Write media files to word/media/
            for (filename, bytes) in &media_files {
                parts.insert(format!("word/media/{}", filename), bytes.clone());

                // Add content type for the image
                let extension = filename.rsplit('.').next().unwrap_or("png");
                let mime_type = match extension {
                    "png" => "image/png",
                    "jpg" | "jpeg" => "image/jpeg",
                    "gif" => "image/gif",
                    "svg" => "image/svg+xml",
                    _ => "image/png",
                };
                content_types_xml = insert_default(&content_types_xml, extension, mime_type)?;
            }

            // Create relationship files for headers/footers that reference images
            // Each header/footer gets its own _rels file if it has images
            if has_header {
                let rels_content = build_header_footer_rels(&media_files);
                parts.insert(
                    "word/_rels/header1.xml.rels".to_string(),
                    rels_content.into_bytes(),
                );
            }
            if has_first_header {
                let rels_content = build_header_footer_rels(&media_files);
                parts.insert(
                    "word/_rels/header2.xml.rels".to_string(),
                    rels_content.into_bytes(),
                );
            }
            if has_footer {
                let rels_content = build_header_footer_rels(&media_files);
                parts.insert(
                    "word/_rels/footer1.xml.rels".to_string(),
                    rels_content.into_bytes(),
                );
            }
            if has_first_footer {
                let rels_content = build_header_footer_rels(&media_files);
                parts.insert(
                    "word/_rels/footer2.xml.rels".to_string(),
                    rels_content.into_bytes(),
                );
            }
        }

        // Update section properties in document.xml to apply page layout and reference headers/footers
        if let Some(document_bytes) = parts.get(DOCUMENT) {
            let mut document_xml = String::from_utf8(document_bytes.clone())?;

            // Build new section properties with page layout and header/footer references
            let new_sect_pr = build_section_properties(
                &resolved_variables,
                has_header,
                has_footer,
                has_first_header,
                has_first_footer,
            );

            // Replace existing w:sectPr with our new one
            // The sectPr is typically at the end of the document, inside w:body
            document_xml = replace_section_properties(&document_xml, &new_sect_pr)?;

            parts.insert(DOCUMENT.to_string(), document_xml.into_bytes());
        }
    }

    // Replace updated XML parts in the HashMap.
    parts.insert(CONTENT_TYPES.to_string(), content_types_xml.into_bytes());
    parts.insert(CUSTOM_PROPS.to_string(), custom_props_xml.into_bytes());
    parts.insert(DOCUMENT_RELS.to_string(), rels_xml.into_bytes());

    // Re-assemble the DOCX.
    let mut tmp = tempfile::NamedTempFile::new()?;
    let mut writer = ZipWriter::new(&mut tmp);
    let opts = SimpleFileOptions::default();

    for (name, data) in parts {
        writer.start_file(name, opts)?;
        writer.write_all(&data)?;
    }
    writer.finish()?;

    move_file(tmp.path(), path)?;

    Ok(())
}

/// Insert a `<Default …/>` into `[Content_Types].xml` unless one for `extension` already exists
pub fn insert_default(xml: &str, extension: &str, content_type: &str) -> Result<String> {
    // Return early if a Default for this extension is already present
    if xml.contains(&format!(r#"Extension="{extension}""#)) {
        return Ok(xml.to_owned());
    }

    // Build the tag we want to inject
    let tag = format!(r#"<Default Extension="{extension}" ContentType="{content_type}"/>"#);

    // Find the last </Types> and splice the tag in front of it
    let pos = xml
        .rfind("</Types>")
        .ok_or_eyre("[Content_Types].xml is missing </Types> tag")?;

    let mut out = String::with_capacity(xml.len() + tag.len());
    out.push_str(&xml[..pos]);
    out.push_str(&tag);
    out.push_str(&xml[pos..]);
    Ok(out)
}

/// Insert an `<Override …/>` into `[Content_Types].xml` unless one for `part` already exists
pub fn insert_override(xml: &str, part: &str, content_type: &str) -> Result<String> {
    // Return early if an Override for this part is already present
    if xml.contains(&format!(r#"PartName="{part}""#)) {
        return Ok(xml.to_owned());
    }

    // Build the tag we want to inject
    let tag = format!(r#"<Override PartName="{part}" ContentType="{content_type}"/>"#);

    // Find the last </Types> and splice the tag in front of it
    let pos = xml
        .rfind("</Types>")
        .ok_or_eyre("[Content_Types].xml is missing </Types> tag")?;

    let mut out = String::with_capacity(xml.len() + tag.len());
    out.push_str(&xml[..pos]);
    out.push_str(&tag);
    out.push_str(&xml[pos..]);
    Ok(out)
}

/// Append a `<Relationship …/>` to `word/_rels/document.xml.rels` if it isn't there already.
pub fn insert_relationship(xml: &str, target: &str) -> Result<String> {
    // Skip if a Relationship for this target already exists
    if xml.contains(&format!(r#"Target="{target}""#)) {
        return Ok(xml.to_owned());
    }

    // Build a new relationship tag
    let id = format!("rId{}", Uuid::new_v4().simple());
    let rel_type = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/customXml";
    let rel_tag = format!(r#"<Relationship Id="{id}" Type="{rel_type}" Target="{target}"/>"#);

    // Inject it just before the closing </Relationships>
    let pos = xml
        .rfind("</Relationships>")
        .ok_or_eyre("word/_rels/document.xml.rels is missing </Relationships> tag")?;

    let mut out = String::with_capacity(xml.len() + rel_tag.len());
    out.push_str(&xml[..pos]);
    out.push_str(&rel_tag);
    out.push_str(&xml[pos..]);
    Ok(out)
}

/// Append a `<Relationship …/>` with a specific ID to `word/_rels/document.xml.rels` if it isn't there already.
///
/// Similar to `insert_relationship` but allows specifying a custom relationship ID.
/// This is needed for headers/footers which require specific IDs to be referenced from sectPr.
pub fn insert_relationship_with_id(
    xml: &str,
    id: &str,
    rel_type: &str,
    target: &str,
) -> Result<String> {
    // Skip if a Relationship with this ID already exists
    if xml.contains(&format!(r#"Id="{id}""#)) {
        return Ok(xml.to_owned());
    }

    let rel_tag = format!(r#"<Relationship Id="{id}" Type="{rel_type}" Target="{target}"/>"#);

    // Inject it just before the closing </Relationships>
    let pos = xml
        .rfind("</Relationships>")
        .ok_or_eyre("word/_rels/document.xml.rels is missing </Relationships> tag")?;

    let mut out = String::with_capacity(xml.len() + rel_tag.len());
    out.push_str(&xml[..pos]);
    out.push_str(&rel_tag);
    out.push_str(&xml[pos..]);
    Ok(out)
}

/// Add `new_props` (name‒value pairs) to the XML string from
/// `docProps/custom.xml`, using nothing but string operations.
///
/// * Appends a `<property …>` node for every pair just before
///   `</Properties>`.
/// * Generates sequential `pid`s (one higher than the current max).
/// * Escapes `& < > ' "` in both name and value.
///
/// If `new_props` is empty the original `xml` is returned unchanged.
pub fn add_custom_props(xml: &str, new_props: Vec<(String, String)>) -> Result<String> {
    if new_props.is_empty() {
        return Ok(xml.to_owned());
    }

    // Find highest existing pid="…"
    let pid_rx = Regex::new(r#"pid="(\d+)""#)?;
    let max_pid = pid_rx
        .captures_iter(xml)
        .filter_map(|c| c.get(1).and_then(|m| m.as_str().parse::<u32>().ok()))
        .max()
        .unwrap_or(1);
    let mut next_pid = max_pid + 1;

    // Build the block of new <property/> elements
    let mut block = String::new();
    for (name, value) in new_props {
        block.push_str(&format!(
            r#"<property pid="{}" fmtid="{{D5CDD505-2E9C-101B-9397-08002B2CF9AE}}" name="{}"><vt:lpwstr>{}</vt:lpwstr></property>"#,
            next_pid,
            escape_xml(&name),
            escape_xml(&value)
        ));
        next_pid += 1;
    }

    let mut out = String::with_capacity(xml.len() + block.len());
    if let Some(pos) = xml.rfind("</Properties>") {
        // Splice the block in before existing </Properties>
        out.push_str(&xml[..pos]);
        out.push_str(&block);
        out.push_str(&xml[pos..]);
    } else if let Some(pos) = xml.rfind("/>") {
        // No, existing custom properties, so replace the end of the self-closing tag
        out.push_str(&xml[..pos]);
        out.push('>');
        out.push_str(&block);
        out.push_str("</Properties>");
    } else {
        // Fallback to constructing new XML
        out = format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Properties xmlns="http://schemas.openxmlformats.org/officeDocument/2006/custom-properties" xmlns:vt="http://schemas.openxmlformats.org/officeDocument/2006/docPropsVTypes">{block}</Properties>"#
        );
    }

    Ok(out)
}

/// Build relationship file for header/footer with embedded images
///
/// Creates a relationship XML file that maps relationship IDs to media files.
/// Each image gets a relationship entry pointing to its location in word/media/.
///
/// # Arguments
/// * `media_files` - Collection of (filename, bytes) for embedded images
///
/// # Returns
/// Complete XML content for a header/footer .rels file
fn build_header_footer_rels(media_files: &[(String, Vec<u8>)]) -> String {
    let mut xml = String::from(
        r#"<?xml version="1.0" encoding="UTF-8"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">"#,
    );

    for (idx, (filename, _)) in media_files.iter().enumerate() {
        let rel_id = idx + 1;
        xml.push_str(&format!(
            r#"<Relationship Id="rId{}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image" Target="media/{}"/>"#,
            rel_id, filename
        ));
    }

    xml.push_str("</Relationships>");
    xml
}

/// Replace the section properties (w:sectPr) in a Word document.xml
///
/// Finds the existing `<w:sectPr>...</w:sectPr>` element and replaces it with `new_sect_pr`.
/// The sectPr is typically at the end of the document, inside `<w:body>`.
///
/// # Arguments
/// * `document_xml` - The complete document.xml content
/// * `new_sect_pr` - The new section properties XML to insert
///
/// # Returns
/// Modified document.xml with replaced section properties
fn replace_section_properties(document_xml: &str, new_sect_pr: &str) -> Result<String> {
    // Find the start of w:sectPr
    let sect_pr_start = document_xml
        .rfind("<w:sectPr")
        .ok_or_eyre("document.xml is missing <w:sectPr element")?;

    // Find the end of w:sectPr (the closing tag)
    let sect_pr_end = document_xml[sect_pr_start..]
        .find("</w:sectPr>")
        .ok_or_eyre("document.xml is missing </w:sectPr> closing tag")?
        + sect_pr_start
        + "</w:sectPr>".len();

    // Build the new document with replaced sectPr
    let mut out = String::with_capacity(document_xml.len() + new_sect_pr.len());
    out.push_str(&document_xml[..sect_pr_start]);
    out.push_str(new_sect_pr);
    out.push_str(&document_xml[sect_pr_end..]);

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_max_media_number_empty() {
        let parts = BTreeMap::new();
        assert_eq!(find_max_media_number(&parts), 0);
    }

    #[test]
    fn test_find_max_media_number_single_file() {
        let mut parts = BTreeMap::new();
        parts.insert("word/media/image1.png".to_string(), vec![]);
        assert_eq!(find_max_media_number(&parts), 1);
    }

    #[test]
    fn test_find_max_media_number_multiple_files() {
        let mut parts = BTreeMap::new();
        parts.insert("word/media/image1.png".to_string(), vec![]);
        parts.insert("word/media/image3.jpg".to_string(), vec![]);
        parts.insert("word/media/image5.svg".to_string(), vec![]);
        assert_eq!(find_max_media_number(&parts), 5);
    }

    #[test]
    fn test_find_max_media_number_non_matching_files() {
        let mut parts = BTreeMap::new();
        parts.insert("word/media/logo.png".to_string(), vec![]);
        parts.insert("word/document.xml".to_string(), vec![]);
        assert_eq!(find_max_media_number(&parts), 0);
    }

    #[test]
    fn test_find_max_media_number_mixed_files() {
        let mut parts = BTreeMap::new();
        parts.insert("word/media/image2.png".to_string(), vec![]);
        parts.insert("word/media/logo.png".to_string(), vec![]);
        parts.insert("word/media/image10.jpg".to_string(), vec![]);
        parts.insert("word/document.xml".to_string(), vec![]);
        assert_eq!(find_max_media_number(&parts), 10);
    }
}
