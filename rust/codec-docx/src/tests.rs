use std::io::{Read, Write};

use indexmap::IndexMap;
use pretty_assertions::assert_eq;
use regex::Regex;
use serde_json::json;
use tempfile::tempdir;
use zip::{ZipArchive, ZipWriter, write::SimpleFileOptions};

use stencila_codec::{
    Codec, EncodeOptions,
    eyre::{Result, bail},
    stencila_schema::{
        Article, ArticleOptions, Inline, Node, Object, Paragraph, Primitive,
        shortcuts::{cc, ce, p, t},
    },
};
use stencila_version::STENCILA_VERSION;

use crate::DocxCodec;
use crate::preprocess::restore_verbatim_char_style;

#[tokio::test]
async fn roundtrip_basic() -> Result<()> {
    let article = Node::Article(Article {
        content: vec![
            // Content including nodes that are reconstituted (e.g. code chunk)
            // These paragraphs with code expressions in different positions are regression tests
            p([ce("0", None::<&str>)]),
            p([t("Before "), ce("1 + 2", Some("r"))]),
            p([t("Before "), ce("3 + 4", Some("python")), t(" after.")]),
            cc("plot(data)", Some("r")),
        ],
        options: Box::new(ArticleOptions {
            // Things that are placed in custom properties
            repository: Some("repository".into()),
            path: Some("path".into()),
            commit: Some("commit".into()),
            extra: Some(Object(IndexMap::from([
                (
                    "generator".into(),
                    Primitive::String(format!("Stencila {STENCILA_VERSION}")),
                ),
                ("boolean".into(), Primitive::Boolean(true)),
                ("integer".into(), Primitive::Integer(123)),
                ("number".into(), Primitive::Number(1.23)),
                ("string".into(), Primitive::String("a string".into())),
                ("array".into(), serde_json::from_value(json!([1, 2, 3]))?),
                (
                    "object".into(),
                    serde_json::from_value(json!({"a": 1, "b": 2}))?,
                ),
            ]))),
            ..Default::default()
        }),
        ..Default::default()
    });

    let temp_dir = tempdir()?;
    let path = temp_dir.path().join("temp.docx");

    DocxCodec
        .to_path(
            &article,
            &path,
            Some(EncodeOptions {
                reproducible: Some(true),
                ..Default::default()
            }),
        )
        .await?;

    let (mut round_tripped, ..) = DocxCodec.from_path(&path, None).await?;

    // Strip the encoding options inserted into extra
    if let Node::Article(Article { options, .. }) = &mut round_tripped
        && let Some(extra) = &mut options.extra
    {
        extra.swap_remove("encoding");
    };

    assert_eq!(round_tripped, article);

    Ok(())
}

/// Test that preprocessing restores inline code from monospace fonts
///
/// This simulates what happens when a DOCX is edited in Google Docs:
/// 1. Create a DOCX with inline code (which gets VerbatimChar style)
/// 2. Strip the VerbatimChar style (simulating Google Docs)
/// 3. Run preprocessing to restore the style
/// 4. Decode and verify inline code is preserved
#[tokio::test]
async fn preprocessing() -> Result<()> {
    use stencila_codec::stencila_schema::{Block, CodeInline, Text};

    // Create an article with inline code
    let article = Node::Article(Article {
        content: vec![Block::Paragraph(Paragraph {
            content: vec![
                Inline::Text(Text {
                    value: "Some code ".into(),
                    ..Default::default()
                }),
                Inline::CodeInline(CodeInline {
                    code: "1 + 2".into(),
                    ..Default::default()
                }),
                Inline::Text(Text {
                    value: ".".into(),
                    ..Default::default()
                }),
            ],
            ..Default::default()
        })],
        ..Default::default()
    });

    let temp_dir = tempdir()?;
    let original_path = temp_dir.path().join("original.docx");
    let modified_path = temp_dir.path().join("modified.docx");

    // Encode to DOCX
    DocxCodec.to_path(&article, &original_path, None).await?;

    // Read the DOCX, strip VerbatimChar references from document.xml (simulating Google Docs),
    // and write to a new file
    {
        let mut docx = std::fs::File::open(&original_path)?;
        let mut zip = ZipArchive::new(&mut docx)?;

        let mut parts: std::collections::BTreeMap<String, Vec<u8>> =
            std::collections::BTreeMap::new();
        for index in 0..zip.len() {
            let mut file = zip.by_index(index)?;
            let mut buffer = Vec::with_capacity(file.size() as usize);
            file.read_to_end(&mut buffer)?;
            parts.insert(file.name().to_owned(), buffer);
        }

        // Strip VerbatimChar style references from document.xml and replace with
        // monospace font (simulating what Google Docs does - it loses the style but
        // preserves the visual appearance as a monospace font)
        if let Some(document_bytes) = parts.get("word/document.xml") {
            let document_str = String::from_utf8(document_bytes.clone())?;
            // Replace the rStyle reference with a monospace font (Roboto Mono - what Google Docs uses)
            let rstyle_regex = Regex::new(r#"<w:rStyle\s+w:val="VerbatimChar"\s*/>"#)?;
            let modified = rstyle_regex
                .replace_all(
                    &document_str,
                    r#"<w:rFonts w:ascii="Roboto Mono" w:hAnsi="Roboto Mono"/>"#,
                )
                .to_string();
            parts.insert("word/document.xml".to_string(), modified.into_bytes());
        }

        // Also remove the VerbatimChar style definition from styles.xml
        // (simulating that Google Docs doesn't preserve custom styles)
        if let Some(styles_bytes) = parts.get("word/styles.xml") {
            let styles_str = String::from_utf8(styles_bytes.clone())?;
            // Remove the entire VerbatimChar style element using a simple replacement
            // (this is a simplified approach - real Google Docs export would just not have the style)
            let style_regex =
                Regex::new(r#"<w:style[^>]*w:styleId="VerbatimChar"[^>]*>.*?</w:style>"#)?;
            let modified = style_regex.replace(&styles_str, "").to_string();
            parts.insert("word/styles.xml".to_string(), modified.into_bytes());
        }

        // Write modified DOCX
        let mut modified_file = std::fs::File::create(&modified_path)?;
        let mut writer = ZipWriter::new(&mut modified_file);
        let opts = SimpleFileOptions::default();

        for (name, data) in parts {
            writer.start_file(name, opts)?;
            writer.write_all(&data)?;
        }
        writer.finish()?;
    }

    // Without preprocessing, the inline code would be lost
    let (decoded_without_preprocess, ..) = DocxCodec.from_path(&modified_path, None).await?;
    if let Node::Article(Article { content, .. }) = &decoded_without_preprocess
        && let Some(block) = content.first()
        && let stencila_codec::stencila_schema::Block::Paragraph(para) = block
    {
        // Without preprocessing, there should be no CodeInline
        let has_code_inline = para
            .content
            .iter()
            .any(|inline| matches!(inline, Inline::CodeInline(_)));
        assert!(
            !has_code_inline,
            "Without preprocessing, inline code should be lost"
        );
    }

    // Now apply preprocessing
    restore_verbatim_char_style(&modified_path)?;

    // After preprocessing, the inline code should be restored
    let (decoded_with_preprocess, ..) = DocxCodec.from_path(&modified_path, None).await?;
    if let Node::Article(Article { content, .. }) = &decoded_with_preprocess
        && let Some(block) = content.first()
        && let stencila_codec::stencila_schema::Block::Paragraph(para) = block
    {
        // With preprocessing, there should be a CodeInline
        let Some(code_inline) = para.content.iter().find_map(|inline| {
            if let Inline::CodeInline(ci) = inline {
                Some(ci)
            } else {
                None
            }
        }) else {
            bail!("With preprocessing, inline code should be restored");
        };

        assert_eq!(code_inline.code.to_string(), "1 + 2");
    }

    Ok(())
}
