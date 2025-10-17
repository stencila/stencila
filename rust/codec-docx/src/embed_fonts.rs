use std::collections::BTreeMap;

use futures::future::join_all;
use serde_json::Value;
use stencila_codec::eyre::{Context, OptionExt, Result};
use stencila_fonts::Font;
use uuid::Uuid;

use crate::encode::{escape_xml, insert_override};

/// Font variables that are actually used in DOCX styles.xml
///
/// Only these fonts will be resolved and embedded to improve performance.
/// This needs to be updated when `encode_theme.rs` uses an additional font.
const DOCX_FONT_VARIABLES: &[&str] = &[
    "text-font-family",
    "heading-font-family",
    "code-font-family",
];

/// Information about a resolved font ready for embedding
pub(crate) struct ResolvedFont {
    /// The font family name (e.g., "Noto Serif")
    pub family: String,

    /// The TTF file bytes
    pub bytes: Vec<u8>,
}

/// Resolve fonts from theme variables by parsing CSS font stacks
///
/// Only resolves fonts that are actually used in DOCX styles (whitelist) and
/// resolves them in parallel for performance. Non-whitelisted fonts are skipped.
///
/// Returns a vector of resolved fonts and an updated variables map where
/// CSS font stacks have been replaced with the resolved font family names.
pub(crate) async fn resolve_fonts(
    variables: &BTreeMap<String, Value>,
) -> Result<(Vec<ResolvedFont>, BTreeMap<String, Value>)> {
    let mut updated_vars = variables.clone();

    // Collect font resolution tasks for whitelisted fonts only
    let mut resolution_tasks = Vec::new();

    for (name, value) in variables {
        if !DOCX_FONT_VARIABLES.contains(&name.as_str()) {
            continue;
        }

        if let Some(stack) = value.as_str() {
            let name = name.clone();
            let stack = stack.to_string();

            resolution_tasks.push(async move {
                let result = Font::resolve_first(&stack).await;
                (name, stack, result)
            });
        }
    }

    // Resolve all fonts concurrently
    let results = join_all(resolution_tasks).await;

    // Process results and collect resolved fonts
    let mut resolved_fonts = Vec::new();

    for (name, stack, result) in results {
        match result {
            Ok(Some(font)) => {
                let family = font.family().to_string();

                // Only embed fonts from safe sources (Google Fonts)
                // Skip system fonts which may be proprietary and violate licenses when embedded
                if !font.is_safe_to_embed() {
                    tracing::debug!(
                        "Skipping embedding of system font '{family}' for {name} (may be proprietary)"
                    );
                    // Still update the variable to use the single resolved font name
                    // Word will find it in system fonts since we're not embedding it
                    updated_vars.insert(name, Value::String(family));
                    continue;
                }

                let bytes = font
                    .bytes()
                    .wrap_err_with(|| format!("Failed to read font bytes for '{family}'"))?;

                tracing::debug!(
                    "Resolved font for {name}: {family} ({} bytes, source: {:?})",
                    bytes.len(),
                    font.source()
                );

                // Check if we already have this font
                if !resolved_fonts
                    .iter()
                    .any(|f: &ResolvedFont| f.family == family)
                {
                    resolved_fonts.push(ResolvedFont {
                        family: family.clone(),
                        bytes,
                    });
                }

                // Update the variable to use the resolved font family name
                updated_vars.insert(name, Value::String(family));
            }
            Ok(None) => {
                // OPTIMIZATION 2: Reduce logging - debug instead of warn
                tracing::debug!("No font found for {name}: {stack}, font will not be embedded");
            }
            Err(error) => {
                // OPTIMIZATION 2: Reduce logging - debug instead of warn
                tracing::debug!(
                    "Error resolving font for {name}: {error}, font will not be embedded"
                );
            }
        }
    }

    Ok((resolved_fonts, updated_vars))
}

/// Embed fonts into the DOCX parts
///
/// This function:
/// 1. Adds font files to the `word/fonts/` directory
/// 2. Generates or updates `word/fontTable.xml` with embedded font references
/// 3. Creates `word/_rels/fontTable.xml.rels` with font relationships
/// 4. Updates `[Content_Types].xml` with font MIME types
/// 5. Adds fontTable relationship to `word/_rels/document.xml.rels`
pub(crate) fn embed_fonts(
    parts: &mut BTreeMap<String, Vec<u8>>,
    fonts: &[ResolvedFont],
    content_types_xml: &mut String,
    document_rels_xml: &mut String,
) -> Result<()> {
    if fonts.is_empty() {
        return Ok(());
    }

    // Build fontTable.xml with embedded font references
    let mut font_table_xml = String::with_capacity(2048);
    font_table_xml.push_str(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:fonts xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">"#,
    );

    // Build fontTable.xml.rels
    let mut font_rels_xml = String::with_capacity(1024);
    font_rels_xml.push_str(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">"#,
    );

    // Process each font
    for (index, font) in fonts.iter().enumerate() {
        let rel_id = index + 1;
        let font_filename = format!("font{}.ttf", rel_id);
        let font_part = format!("word/fonts/{}", font_filename);

        // Add font file to parts
        parts.insert(font_part.clone(), font.bytes.clone());

        // Add font entry to fontTable.xml
        font_table_xml.push_str(&format!(
            r#"<w:font w:name="{}"><w:embedRegular r:id="rId{}"/><w:family w:val="auto"/></w:font>"#,
            escape_xml(&font.family),
            rel_id
        ));

        // Add relationship to fontTable.xml.rels
        font_rels_xml.push_str(&format!(
            r#"<Relationship Id="rId{}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/font" Target="fonts/{}"/>"#,
            rel_id,
            font_filename
        ));

        // Add Override to [Content_Types].xml
        *content_types_xml = insert_override(
            content_types_xml,
            &format!("/{}", font_part),
            "application/x-font-ttf",
        )?;
    }

    font_table_xml.push_str("</w:fonts>");
    font_rels_xml.push_str("</Relationships>");

    // Add fontTable.xml to parts
    const FONT_TABLE: &str = "word/fontTable.xml";
    parts.insert(FONT_TABLE.to_string(), font_table_xml.into_bytes());

    // Add fontTable.xml.rels to parts
    const FONT_TABLE_RELS: &str = "word/_rels/fontTable.xml.rels";
    parts.insert(FONT_TABLE_RELS.to_string(), font_rels_xml.into_bytes());

    // Add Override for fontTable.xml to [Content_Types].xml
    *content_types_xml = insert_override(
        content_types_xml,
        "/word/fontTable.xml",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.fontTable+xml",
    )?;

    // Add fontTable relationship to document.xml.rels if not already present
    if !document_rels_xml.contains("fontTable.xml") {
        *document_rels_xml = insert_font_table_relationship(document_rels_xml)?;
    }

    Ok(())
}

/// Insert a relationship to fontTable.xml in document.xml.rels
fn insert_font_table_relationship(xml: &str) -> Result<String> {
    // Check if relationship already exists
    if xml.contains(r#"Target="fontTable.xml""#) {
        return Ok(xml.to_owned());
    }

    // Build the relationship tag
    let id = format!("rId{}", Uuid::new_v4().simple());
    let rel_type = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/fontTable";
    let rel_tag = format!(r#"<Relationship Id="{id}" Type="{rel_type}" Target="fontTable.xml"/>"#);

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
