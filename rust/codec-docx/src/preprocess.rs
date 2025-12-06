//! Pre-processing of DOCX files downloaded from external sources
//!
//! This module provides functionality to restore formatting that may be lost
//! when documents are edited in external applications like Google Docs.

use std::{
    collections::BTreeMap,
    fs::File,
    io::{Read, Write},
    path::Path,
};

use regex::Regex;
use zip::{ZipArchive, ZipWriter, write::SimpleFileOptions};

use stencila_codec::eyre::{Context, Result, eyre};
use stencila_codec_utils::move_file;

/// Restore VerbatimChar style to text runs with monospace fonts
///
/// Google Docs doesn't preserve character styles like "Verbatim Char" that
/// Pandoc uses to mark inline code. This function scans the document for
/// text runs using monospace fonts and adds the VerbatimChar style reference.
///
/// This should be called on DOCX files downloaded from Google Docs before
/// decoding with Pandoc.
pub fn restore_verbatim_char_style(path: &Path) -> Result<()> {
    // Open DOCX as ZIP
    let mut docx =
        File::open(path).wrap_err_with(|| eyre!("unable to open: {}", path.display()))?;
    let mut zip = ZipArchive::new(&mut docx)
        .wrap_err_with(|| eyre!("DOCX is not a valid zip: {}", path.display()))?;

    // Read all parts into memory
    let mut parts: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    for index in 0..zip.len() {
        let mut file = zip.by_index(index)?;
        let mut buffer = Vec::with_capacity(file.size() as usize);
        file.read_to_end(&mut buffer)?;
        parts.insert(file.name().to_owned(), buffer);
    }

    // Check if document.xml exists
    let document_xml = match parts.get("word/document.xml") {
        Some(bytes) => String::from_utf8(bytes.clone())?,
        None => return Ok(()), // No document.xml, nothing to do
    };

    // Process document.xml to add VerbatimChar style to monospace runs
    let modified_document = add_verbatim_style_to_monospace_runs(&document_xml)?;
    let document_changed = modified_document != document_xml;

    // Only proceed if we made changes
    if !document_changed {
        return Ok(());
    }

    parts.insert(
        "word/document.xml".to_string(),
        modified_document.into_bytes(),
    );

    // Ensure VerbatimChar style exists in styles.xml
    if let Some(styles_bytes) = parts.get("word/styles.xml") {
        let styles_str = String::from_utf8(styles_bytes.clone())?;
        if !styles_str.contains("VerbatimChar") {
            let modified_styles = inject_verbatim_char_style(&styles_str)?;
            parts.insert("word/styles.xml".to_string(), modified_styles.into_bytes());
        }
    }

    // Re-assemble the DOCX
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

/// Check if a font name is a monospace font
fn is_monospace_font(font_name: &str) -> bool {
    /// Known monospace fonts that should be treated as inline code
    ///
    /// Font names are lowercase for case-insensitive matching.
    const MONOSPACE_FONTS: &[&str] = &[
        // Microsoft fonts
        "consolas",
        "courier new",
        "lucida console",
        // Apple fonts
        "menlo",
        "monaco",
        "andale mono",
        "sf mono",
        // Open source monospace fonts
        "dejavu sans mono",
        "liberation mono",
        "fira code",
        "fira mono",
        "jetbrains mono",
        "source code pro",
        "ubuntu mono",
        "roboto mono",
        "noto sans mono",
        "inconsolata",
        "droid sans mono",
        "cascadia code",
        "cascadia mono",
        "ibm plex mono",
        "pt mono",
        "dm mono",
        "space mono",
        "overpass mono",
        "red hat mono",
        "freemono",
        "bitstream vera sans mono",
        // Classic fonts
        "courier",
        "lucida sans typewriter",
        // Other common monospace
        "hack",
        "anonymous pro",
        "input mono",
        "iosevka",
    ];

    let normalized = font_name.to_lowercase();
    MONOSPACE_FONTS.iter().any(|&mono| {
        // Exact match or font name starts with the monospace name
        // This handles variants like "Consolas Bold" or "Menlo-Regular"
        normalized == mono || normalized.starts_with(&format!("{mono} "))
    })
}

/// Add VerbatimChar style reference to text runs with monospace fonts
///
/// Finds `<w:r>` elements whose `<w:rPr>` contains a `<w:rFonts>` with a
/// monospace font in the `w:ascii` attribute, and adds `<w:rStyle w:val="VerbatimChar"/>`
/// if no style is already present.
fn add_verbatim_style_to_monospace_runs(document_xml: &str) -> Result<String> {
    // Regex to match runs: <w:r...><w:rPr>...</w:rPr>
    // We need to be careful because runs can be complex
    // This regex captures:
    // 1. The opening <w:r> or <w:r ...> tag
    // 2. The <w:rPr> opening tag
    // 3. The content inside rPr (non-greedy)
    // 4. The </w:rPr> closing tag
    //
    // Using (?s) for DOTALL mode to match across newlines
    let run_pattern = Regex::new(r#"(?s)(<w:r(?:\s[^>]*)?>)\s*(<w:rPr>)(.*?)(</w:rPr>)"#)?;

    // Regex to extract the w:ascii font name from rFonts
    let font_pattern = Regex::new(r#"<w:rFonts[^>]*\bw:ascii="([^"]+)"[^>]*/>"#)?;

    let result = run_pattern.replace_all(document_xml, |caps: &regex::Captures| {
        let run_start = &caps[1];
        let rpr_start = &caps[2];
        let rpr_content = &caps[3];
        let rpr_end = &caps[4];

        // Skip if already has a style reference
        if rpr_content.contains("<w:rStyle") {
            return caps[0].to_string();
        }

        // Check for monospace font in w:ascii attribute
        if let Some(font_caps) = font_pattern.captures(rpr_content) {
            let font_name = &font_caps[1];
            if is_monospace_font(font_name) {
                // Inject VerbatimChar style at the beginning of rPr content
                return format!(
                    "{}{}<w:rStyle w:val=\"VerbatimChar\"/>{}{}",
                    run_start, rpr_start, rpr_content, rpr_end
                );
            }
        }

        caps[0].to_string()
    });

    Ok(result.into_owned())
}

/// Inject the VerbatimChar style definition into styles.xml
fn inject_verbatim_char_style(styles_xml: &str) -> Result<String> {
    /// The minimal VerbatimChar style definition to inject if not present
    const VERBATIM_CHAR_STYLE: &str = r#"<w:style w:type="character" w:styleId="VerbatimChar" w:customStyle="1"><w:name w:val="Verbatim Char"/><w:rPr></w:rPr></w:style>"#;

    // Find </w:styles> and insert the VerbatimChar style before it
    if let Some(pos) = styles_xml.rfind("</w:styles>") {
        let mut result = String::with_capacity(styles_xml.len() + VERBATIM_CHAR_STYLE.len());
        result.push_str(&styles_xml[..pos]);
        result.push_str(VERBATIM_CHAR_STYLE);
        result.push_str(&styles_xml[pos..]);
        Ok(result)
    } else {
        // If we can't find the closing tag, return unchanged
        Ok(styles_xml.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_monospace_font() {
        // Exact matches (case-insensitive)
        assert!(is_monospace_font("Consolas"));
        assert!(is_monospace_font("consolas"));
        assert!(is_monospace_font("CONSOLAS"));
        assert!(is_monospace_font("Courier New"));
        assert!(is_monospace_font("Menlo"));
        assert!(is_monospace_font("Monaco"));
        assert!(is_monospace_font("Fira Code"));
        assert!(is_monospace_font("JetBrains Mono"));
        assert!(is_monospace_font("Noto Sans Mono"));

        // Variants with suffixes
        assert!(is_monospace_font("Consolas Bold"));
        assert!(is_monospace_font("Menlo Regular"));

        // Non-monospace fonts
        assert!(!is_monospace_font("Arial"));
        assert!(!is_monospace_font("Times New Roman"));
        assert!(!is_monospace_font("Calibri"));
        assert!(!is_monospace_font("Helvetica"));
        assert!(!is_monospace_font("Georgia"));
    }

    #[test]
    fn test_add_verbatim_style_basic() -> Result<()> {
        let input = r#"<w:r><w:rPr><w:rFonts w:ascii="Consolas" w:hAnsi="Consolas"/></w:rPr><w:t>code</w:t></w:r>"#;
        let output = add_verbatim_style_to_monospace_runs(input)?;

        assert!(output.contains(r#"<w:rStyle w:val="VerbatimChar"/>"#));
        assert!(output.contains(r#"<w:rFonts w:ascii="Consolas""#));
        Ok(())
    }

    #[test]
    fn test_add_verbatim_style_preserves_existing_style() -> Result<()> {
        let input = r#"<w:r><w:rPr><w:rStyle w:val="SomeStyle"/><w:rFonts w:ascii="Consolas"/></w:rPr><w:t>code</w:t></w:r>"#;
        let output = add_verbatim_style_to_monospace_runs(input)?;

        // Should not add another rStyle
        assert_eq!(output.matches("<w:rStyle").count(), 1);
        assert!(output.contains(r#"<w:rStyle w:val="SomeStyle"/>"#));
        Ok(())
    }

    #[test]
    fn test_add_verbatim_style_ignores_non_monospace() -> Result<()> {
        let input = r#"<w:r><w:rPr><w:rFonts w:ascii="Arial" w:hAnsi="Arial"/></w:rPr><w:t>text</w:t></w:r>"#;
        let output = add_verbatim_style_to_monospace_runs(input)?;

        assert!(!output.contains("VerbatimChar"));
        assert_eq!(input, output);
        Ok(())
    }

    #[test]
    fn test_add_verbatim_style_multiple_runs() -> Result<()> {
        let input = r#"<w:p><w:r><w:rPr><w:rFonts w:ascii="Arial"/></w:rPr><w:t>normal</w:t></w:r><w:r><w:rPr><w:rFonts w:ascii="Consolas"/></w:rPr><w:t>code</w:t></w:r><w:r><w:rPr><w:rFonts w:ascii="Menlo"/></w:rPr><w:t>more code</w:t></w:r></w:p>"#;
        let output = add_verbatim_style_to_monospace_runs(input)?;

        // Should add VerbatimChar to both monospace runs (Consolas and Menlo)
        assert_eq!(output.matches("VerbatimChar").count(), 2);
        Ok(())
    }

    #[test]
    fn test_inject_verbatim_char_style() -> Result<()> {
        let input = r#"<?xml version="1.0"?><w:styles xmlns:w="..."><w:style w:styleId="Normal"/></w:styles>"#;
        let output = inject_verbatim_char_style(input)?;

        assert!(output.contains("VerbatimChar"));
        assert!(output.contains(r#"<w:name w:val="Verbatim Char"/>"#));
        assert!(output.ends_with("</w:styles>"));
        Ok(())
    }
}
