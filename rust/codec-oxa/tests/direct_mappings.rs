//! Tests for direct type encode/decode for Stencila/OXA type pairs, round-trip
//! fidelity, root decode error handling, and codec registration/dispatch.

use pretty_assertions::assert_eq;
use serde_json::Value;
use stencila_codec::{
    Codec,
    eyre::{self, OptionExt, Result},
    stencila_schema::{
        Article, Block, Inline, Node,
        shortcuts::{art, cb, ci, em, h, p, stg, sub, sup, t, tb},
    },
};
use stencila_codec_oxa::OxaCodec;

// ---------------------------------------------------------------------------
// Codec registration and dispatch tests
// ---------------------------------------------------------------------------

/// OxaCodec reports its name as "oxa"
#[test]
fn codec_name() {
    let codec = OxaCodec;
    assert_eq!(codec.name(), "oxa");
}

/// OxaCodec supports encoding to the OXA format
#[test]
fn codec_supports_to_oxa_format() {
    let codec = OxaCodec;
    let format = stencila_codec::stencila_format::Format::from_name("oxa");
    let support = codec.supports_to_format(&format);
    assert!(
        support.is_supported(),
        "OxaCodec should support encoding to the OXA format, got {support:?}"
    );
}

/// OxaCodec supports decoding from the OXA format
#[test]
fn codec_supports_from_oxa_format() {
    let codec = OxaCodec;
    let format = stencila_codec::stencila_format::Format::from_name("oxa");
    let support = codec.supports_from_format(&format);
    assert!(
        support.is_supported(),
        "OxaCodec should support decoding from the OXA format, got {support:?}"
    );
}

/// OxaCodec does not support unrelated formats
#[test]
fn codec_does_not_support_other_formats() {
    let codec = OxaCodec;
    assert!(
        !codec
            .supports_to_format(&stencila_codec::stencila_format::Format::Json)
            .is_supported(),
        "OxaCodec should not support encoding to JSON format"
    );
    assert!(
        !codec
            .supports_from_format(&stencila_codec::stencila_format::Format::Html)
            .is_supported(),
        "OxaCodec should not support decoding from HTML format"
    );
}

// ---------------------------------------------------------------------------
// Encode tests: Stencila → OXA JSON
// ---------------------------------------------------------------------------

/// Encode a simple article with a single paragraph containing plain text
#[tokio::test]
async fn encode_article_with_paragraph() -> Result<()> {
    let codec = OxaCodec;
    let doc = art([p([t("Hello world")])]);

    let (json_str, _info) = codec.to_string(&doc, None).await?;
    let value: Value = serde_json::from_str(&json_str)?;

    assert_eq!(value["type"], "Document");
    assert_eq!(value["children"][0]["type"], "Paragraph");
    assert_eq!(value["children"][0]["children"][0]["type"], "Text");
    assert_eq!(value["children"][0]["children"][0]["value"], "Hello world");

    Ok(())
}

/// Encode headings with level mapping
#[tokio::test]
async fn encode_heading() -> Result<()> {
    let codec = OxaCodec;
    let doc = art([h(2, [t("My Heading")])]);

    let (json_str, _info) = codec.to_string(&doc, None).await?;
    let value: Value = serde_json::from_str(&json_str)?;

    let heading = &value["children"][0];
    assert_eq!(heading["type"], "Heading");
    assert_eq!(heading["level"], 2);
    assert_eq!(heading["children"][0]["type"], "Text");
    assert_eq!(heading["children"][0]["value"], "My Heading");

    Ok(())
}

/// Encode emphasis (italic) inline
#[tokio::test]
async fn encode_emphasis() -> Result<()> {
    let codec = OxaCodec;
    let doc = art([p([em([t("italic text")])])]);

    let (json_str, _info) = codec.to_string(&doc, None).await?;
    let value: Value = serde_json::from_str(&json_str)?;

    let inline = &value["children"][0]["children"][0];
    assert_eq!(inline["type"], "Emphasis");
    assert_eq!(inline["children"][0]["type"], "Text");
    assert_eq!(inline["children"][0]["value"], "italic text");

    Ok(())
}

/// Encode strong (bold) inline
#[tokio::test]
async fn encode_strong() -> Result<()> {
    let codec = OxaCodec;
    let doc = art([p([stg([t("bold text")])])]);

    let (json_str, _info) = codec.to_string(&doc, None).await?;
    let value: Value = serde_json::from_str(&json_str)?;

    let inline = &value["children"][0]["children"][0];
    assert_eq!(inline["type"], "Strong");
    assert_eq!(inline["children"][0]["type"], "Text");
    assert_eq!(inline["children"][0]["value"], "bold text");

    Ok(())
}

/// Encode code block with language
#[tokio::test]
async fn encode_code_block() -> Result<()> {
    let codec = OxaCodec;
    let doc = art([cb("print('hello')", Some("python"))]);

    let (json_str, _info) = codec.to_string(&doc, None).await?;
    let value: Value = serde_json::from_str(&json_str)?;

    let code = &value["children"][0];
    assert_eq!(code["type"], "Code");
    assert_eq!(code["value"], "print('hello')");
    assert_eq!(code["language"], "python");

    Ok(())
}

/// Encode inline code
#[tokio::test]
async fn encode_code_inline() -> Result<()> {
    let codec = OxaCodec;
    let doc = art([p([ci("x = 1")])]);

    let (json_str, _info) = codec.to_string(&doc, None).await?;
    let value: Value = serde_json::from_str(&json_str)?;

    let inline = &value["children"][0]["children"][0];
    assert_eq!(inline["type"], "InlineCode");
    assert_eq!(inline["value"], "x = 1");

    Ok(())
}

/// Encode subscript inline
#[tokio::test]
async fn encode_subscript() -> Result<()> {
    let codec = OxaCodec;
    let doc = art([p([sub([t("2")])])]);

    let (json_str, _info) = codec.to_string(&doc, None).await?;
    let value: Value = serde_json::from_str(&json_str)?;

    let inline = &value["children"][0]["children"][0];
    assert_eq!(inline["type"], "Subscript");
    assert_eq!(inline["children"][0]["type"], "Text");
    assert_eq!(inline["children"][0]["value"], "2");

    Ok(())
}

/// Encode superscript inline
#[tokio::test]
async fn encode_superscript() -> Result<()> {
    let codec = OxaCodec;
    let doc = art([p([sup([t("n")])])]);

    let (json_str, _info) = codec.to_string(&doc, None).await?;
    let value: Value = serde_json::from_str(&json_str)?;

    let inline = &value["children"][0]["children"][0];
    assert_eq!(inline["type"], "Superscript");
    assert_eq!(inline["children"][0]["type"], "Text");
    assert_eq!(inline["children"][0]["value"], "n");

    Ok(())
}

/// Encode thematic break
#[tokio::test]
async fn encode_thematic_break() -> Result<()> {
    let codec = OxaCodec;
    let doc = art([tb()]);

    let (json_str, _info) = codec.to_string(&doc, None).await?;
    let value: Value = serde_json::from_str(&json_str)?;

    let block = &value["children"][0];
    assert_eq!(block["type"], "ThematicBreak");

    Ok(())
}

/// Encode a complex article with all direct-mapped types together
#[tokio::test]
async fn encode_all_direct_types() -> Result<()> {
    let codec = OxaCodec;
    let doc = art([
        h(1, [t("Title")]),
        p([
            t("Normal "),
            em([t("italic")]),
            t(" "),
            stg([t("bold")]),
            t(" "),
            ci("code"),
            t(" "),
            sub([t("sub")]),
            t(" "),
            sup([t("sup")]),
        ]),
        cb("fn main() {}", Some("rust")),
        tb(),
    ]);

    let (json_str, _info) = codec.to_string(&doc, None).await?;
    let value: Value = serde_json::from_str(&json_str)?;

    assert_eq!(value["type"], "Document");
    let children = value["children"]
        .as_array()
        .ok_or_eyre("children should be an array")?;
    assert_eq!(children.len(), 4);

    assert_eq!(children[0]["type"], "Heading");
    assert_eq!(children[1]["type"], "Paragraph");
    assert_eq!(children[2]["type"], "Code");
    assert_eq!(children[3]["type"], "ThematicBreak");

    let para_children = children[1]["children"]
        .as_array()
        .ok_or_eyre("paragraph children should be an array")?;
    let types: Vec<&str> = para_children
        .iter()
        .filter_map(|c| c["type"].as_str())
        .collect();
    assert_eq!(
        types,
        vec![
            "Text",
            "Emphasis",
            "Text",
            "Strong",
            "Text",
            "InlineCode",
            "Text",
            "Subscript",
            "Text",
            "Superscript",
        ]
    );

    Ok(())
}

/// Encode an article with a title to verify Article↔Document title mapping
#[tokio::test]
async fn encode_article_with_title() -> Result<()> {
    let codec = OxaCodec;

    let doc = Node::Article(Article {
        title: Some(vec![Inline::Text(
            stencila_codec::stencila_schema::Text::new("My Document Title".into()),
        )]),
        content: vec![Block::Paragraph(
            stencila_codec::stencila_schema::Paragraph::new(vec![Inline::Text(
                stencila_codec::stencila_schema::Text::new("Body text".into()),
            )]),
        )],
        ..Default::default()
    });

    let (json_str, _info) = codec.to_string(&doc, None).await?;
    let value: Value = serde_json::from_str(&json_str)?;

    assert_eq!(value["type"], "Document");

    // Article.title should map to Document.title with the exact title text preserved
    let title = &value["title"];
    assert!(
        !title.is_null(),
        "Document should have a title field from Article.title"
    );
    // The title should contain "My Document Title" either as a string or within its structure
    let title_str = title.to_string();
    assert!(
        title_str.contains("My Document Title"),
        "Encoded title should contain the exact text 'My Document Title', got: {title_str}"
    );

    Ok(())
}

// ---------------------------------------------------------------------------
// Decode tests: OXA JSON → Stencila
// ---------------------------------------------------------------------------

/// Decode a simple Document with a paragraph (covers Document→Article, Paragraph, Text)
#[tokio::test]
async fn decode_document_with_paragraph() -> Result<()> {
    let codec = OxaCodec;
    let oxa_json = r#"{
        "type": "Document",
        "children": [
            {
                "type": "Paragraph",
                "children": [
                    {
                        "type": "Text",
                        "value": "Hello world"
                    }
                ]
            }
        ]
    }"#;

    let (node, _info) = codec.from_str(oxa_json, None).await?;

    let expected = art([p([t("Hello world")])]);
    assert_eq!(node, expected);

    Ok(())
}

/// Decode a Heading with level and text content
#[tokio::test]
async fn decode_heading() -> Result<()> {
    let codec = OxaCodec;
    let oxa_json = r#"{
        "type": "Document",
        "children": [
            {
                "type": "Heading",
                "level": 3,
                "children": [
                    {
                        "type": "Text",
                        "value": "Section Title"
                    }
                ]
            }
        ]
    }"#;

    let (node, _info) = codec.from_str(oxa_json, None).await?;

    let expected = art([h(3, [t("Section Title")])]);
    assert_eq!(node, expected);

    Ok(())
}

/// Decode emphasis, strong, subscript, superscript inlines
#[tokio::test]
async fn decode_inline_formatting() -> Result<()> {
    let codec = OxaCodec;
    let oxa_json = r#"{
        "type": "Document",
        "children": [
            {
                "type": "Paragraph",
                "children": [
                    {
                        "type": "Emphasis",
                        "children": [{"type": "Text", "value": "em"}]
                    },
                    {
                        "type": "Strong",
                        "children": [{"type": "Text", "value": "str"}]
                    },
                    {
                        "type": "Subscript",
                        "children": [{"type": "Text", "value": "sub"}]
                    },
                    {
                        "type": "Superscript",
                        "children": [{"type": "Text", "value": "sup"}]
                    }
                ]
            }
        ]
    }"#;

    let (node, _info) = codec.from_str(oxa_json, None).await?;

    let expected = art([p([
        em([t("em")]),
        stg([t("str")]),
        sub([t("sub")]),
        sup([t("sup")]),
    ])]);
    assert_eq!(node, expected);

    Ok(())
}

/// Decode a Code block (OXA Code → Stencila CodeBlock)
#[tokio::test]
async fn decode_code_block() -> Result<()> {
    let codec = OxaCodec;
    let oxa_json = r#"{
        "type": "Document",
        "children": [
            {
                "type": "Code",
                "value": "let x = 1;",
                "language": "rust"
            }
        ]
    }"#;

    let (node, _info) = codec.from_str(oxa_json, None).await?;

    let expected = art([cb("let x = 1;", Some("rust"))]);
    assert_eq!(node, expected);

    Ok(())
}

/// Decode an InlineCode (OXA InlineCode → Stencila CodeInline)
#[tokio::test]
async fn decode_inline_code() -> Result<()> {
    let codec = OxaCodec;
    let oxa_json = r#"{
        "type": "Document",
        "children": [
            {
                "type": "Paragraph",
                "children": [
                    {
                        "type": "InlineCode",
                        "value": "x + 1"
                    }
                ]
            }
        ]
    }"#;

    let (node, _info) = codec.from_str(oxa_json, None).await?;

    let expected = art([p([ci("x + 1")])]);
    assert_eq!(node, expected);

    Ok(())
}

/// Decode a ThematicBreak
#[tokio::test]
async fn decode_thematic_break() -> Result<()> {
    let codec = OxaCodec;
    let oxa_json = r#"{
        "type": "Document",
        "children": [
            {
                "type": "ThematicBreak"
            }
        ]
    }"#;

    let (node, _info) = codec.from_str(oxa_json, None).await?;

    let expected = art([tb()]);
    assert_eq!(node, expected);

    Ok(())
}

/// Decode a standalone plain Text node inside a paragraph
#[tokio::test]
async fn decode_plain_text() -> Result<()> {
    let codec = OxaCodec;
    let oxa_json = r#"{
        "type": "Document",
        "children": [
            {
                "type": "Paragraph",
                "children": [
                    {"type": "Text", "value": "just plain text"}
                ]
            }
        ]
    }"#;

    let (node, _info) = codec.from_str(oxa_json, None).await?;

    let expected = art([p([t("just plain text")])]);
    assert_eq!(node, expected);

    Ok(())
}

/// Decode all 11 direct-mapped types in a single document (parallel to encode_all_direct_types)
#[tokio::test]
async fn decode_all_direct_types() -> Result<()> {
    let codec = OxaCodec;
    let oxa_json = r#"{
        "type": "Document",
        "children": [
            {
                "type": "Heading",
                "level": 1,
                "children": [{"type": "Text", "value": "Title"}]
            },
            {
                "type": "Paragraph",
                "children": [
                    {"type": "Text", "value": "Normal "},
                    {"type": "Emphasis", "children": [{"type": "Text", "value": "italic"}]},
                    {"type": "Text", "value": " "},
                    {"type": "Strong", "children": [{"type": "Text", "value": "bold"}]},
                    {"type": "Text", "value": " "},
                    {"type": "InlineCode", "value": "code"},
                    {"type": "Text", "value": " "},
                    {"type": "Subscript", "children": [{"type": "Text", "value": "sub"}]},
                    {"type": "Text", "value": " "},
                    {"type": "Superscript", "children": [{"type": "Text", "value": "sup"}]}
                ]
            },
            {
                "type": "Code",
                "value": "fn main() {}",
                "language": "rust"
            },
            {
                "type": "ThematicBreak"
            }
        ]
    }"#;

    let (node, _info) = codec.from_str(oxa_json, None).await?;

    let expected = art([
        h(1, [t("Title")]),
        p([
            t("Normal "),
            em([t("italic")]),
            t(" "),
            stg([t("bold")]),
            t(" "),
            ci("code"),
            t(" "),
            sub([t("sub")]),
            t(" "),
            sup([t("sup")]),
        ]),
        cb("fn main() {}", Some("rust")),
        tb(),
    ]);
    assert_eq!(node, expected);

    Ok(())
}

/// Decode a Document with a title to verify exact Article↔Document title mapping
#[tokio::test]
async fn decode_document_with_title() -> Result<()> {
    let codec = OxaCodec;
    let oxa_json = r#"{
        "type": "Document",
        "title": "My Document Title",
        "children": [
            {
                "type": "Paragraph",
                "children": [{"type": "Text", "value": "Body text"}]
            }
        ]
    }"#;

    let (node, _info) = codec.from_str(oxa_json, None).await?;

    let Node::Article(article) = &node else {
        eyre::bail!("Expected Node::Article but got a different variant");
    };

    let title_inlines = article
        .title
        .as_ref()
        .ok_or_eyre("Article.title should be set from Document.title")?;
    assert!(!title_inlines.is_empty(), "Title should not be empty");

    // Verify the exact title text is preserved
    let title_text: String = title_inlines
        .iter()
        .filter_map(|inline| match inline {
            Inline::Text(text) => Some(text.value.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(
        title_text, "My Document Title",
        "Title text should be exactly 'My Document Title'"
    );

    Ok(())
}

/// Decode a Document with metadata to verify metadata fields map into Article
#[tokio::test]
async fn decode_document_with_metadata() -> Result<()> {
    let codec = OxaCodec;
    let oxa_json = r#"{
        "type": "Document",
        "title": "Research Paper",
        "metadata": {
            "doi": "10.1234/example",
            "keywords": ["science", "research"]
        },
        "children": [
            {
                "type": "Paragraph",
                "children": [{"type": "Text", "value": "Content"}]
            }
        ]
    }"#;

    let (node, _info) = codec.from_str(oxa_json, None).await?;

    let Node::Article(article) = &node else {
        eyre::bail!("Expected Node::Article but got a different variant");
    };

    // Verify title mapping from Document.title
    let title_inlines = article
        .title
        .as_ref()
        .ok_or_eyre("Article should have a title from Document.title")?;
    let title_text: String = title_inlines
        .iter()
        .filter_map(|inline| match inline {
            Inline::Text(text) => Some(text.value.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(title_text, "Research Paper");

    // Verify content is correctly decoded
    assert_eq!(article.content.len(), 1);

    // Verify metadata fields are mapped: doi should appear on the Article
    // (The design doc says Article metadata that doesn't have a direct Document field
    // is serialized into Document.metadata, and on decode should map back.)
    assert_eq!(
        article.doi.as_deref(),
        Some("10.1234/example"),
        "Article.doi should be mapped from Document.metadata.doi"
    );

    // Verify keywords are mapped
    let keywords =
        article.options.keywords.as_ref().ok_or_eyre(
            "Article.options.keywords should be mapped from Document.metadata.keywords",
        )?;
    assert_eq!(keywords, &["science", "research"]);

    Ok(())
}

// ---------------------------------------------------------------------------
// Error handling tests: malformed input with descriptive errors
// ---------------------------------------------------------------------------

/// Malformed JSON should produce a descriptive error mentioning JSON parsing
#[tokio::test]
async fn decode_error_malformed_json() -> Result<()> {
    let codec = OxaCodec;
    let result = codec.from_str("not valid json {{{{", None).await;

    assert!(result.is_err(), "Malformed JSON should produce an error");
    let err_msg = match result {
        Err(e) => e.to_string(),
        Ok(_) => unreachable!("already asserted is_err"),
    };
    assert!(
        err_msg.to_lowercase().contains("json")
            || err_msg.to_lowercase().contains("parse")
            || err_msg.to_lowercase().contains("expected"),
        "Error message should mention JSON parsing: {err_msg}"
    );

    Ok(())
}

/// Non-object JSON (e.g. array) should produce a descriptive error mentioning object/root
#[tokio::test]
async fn decode_error_non_object_json() -> Result<()> {
    let codec = OxaCodec;
    let result = codec.from_str("[1, 2, 3]", None).await;

    assert!(result.is_err(), "Non-object JSON should produce an error");
    let err_msg = match result {
        Err(e) => e.to_string(),
        Ok(_) => unreachable!("already asserted is_err"),
    };
    let lower = err_msg.to_lowercase();
    assert!(
        lower.contains("object")
            || lower.contains("root")
            || lower.contains("document")
            || lower.contains("expected")
            || lower.contains("type"),
        "Error message should describe the non-object root issue: {err_msg}"
    );

    Ok(())
}

/// Wrong root type (not "Document") should produce a descriptive error
#[tokio::test]
async fn decode_error_wrong_root_type() -> Result<()> {
    let codec = OxaCodec;
    let oxa_json = r#"{
        "type": "Paragraph",
        "children": [{"type": "Text", "value": "hello"}]
    }"#;

    let result = codec.from_str(oxa_json, None).await;

    assert!(
        result.is_err(),
        "Non-Document root type should produce an error"
    );
    let err_msg = match result {
        Err(e) => e.to_string(),
        Ok(_) => unreachable!("already asserted is_err"),
    };
    assert!(
        err_msg.to_lowercase().contains("document")
            || err_msg.to_lowercase().contains("root")
            || err_msg.to_lowercase().contains("type"),
        "Error message should mention expected Document root type: {err_msg}"
    );

    Ok(())
}

/// JSON object without a "type" field should produce a descriptive error
#[tokio::test]
async fn decode_error_missing_type_field() -> Result<()> {
    let codec = OxaCodec;
    let oxa_json = r#"{"children": []}"#;

    let result = codec.from_str(oxa_json, None).await;

    assert!(
        result.is_err(),
        "Missing type field should produce an error"
    );
    let err_msg = match result {
        Err(e) => e.to_string(),
        Ok(_) => unreachable!("already asserted is_err"),
    };
    let lower = err_msg.to_lowercase();
    assert!(
        lower.contains("type") || lower.contains("document") || lower.contains("missing"),
        "Error message should describe the missing type issue: {err_msg}"
    );

    Ok(())
}

/// A JSON string (not object/array) should produce a descriptive error
#[tokio::test]
async fn decode_error_string_json() -> Result<()> {
    let codec = OxaCodec;
    let result = codec.from_str(r#""just a string""#, None).await;

    assert!(result.is_err(), "String JSON should produce an error");
    let err_msg = match result {
        Err(e) => e.to_string(),
        Ok(_) => unreachable!("already asserted is_err"),
    };
    let lower = err_msg.to_lowercase();
    assert!(
        lower.contains("object")
            || lower.contains("root")
            || lower.contains("document")
            || lower.contains("expected")
            || lower.contains("type"),
        "Error message should describe the non-object root issue: {err_msg}"
    );

    Ok(())
}

// ---------------------------------------------------------------------------
// Round-trip tests: encode → decode → compare
// ---------------------------------------------------------------------------

/// Round-trip a simple article through encode → decode and compare
#[tokio::test]
async fn roundtrip_simple_article() -> Result<()> {
    let codec = OxaCodec;
    let original = art([
        h(1, [t("Title")]),
        p([t("A paragraph with "), em([t("emphasis")]), t(".")]),
        tb(),
    ]);

    let (json_str, _encode_info) = codec.to_string(&original, None).await?;
    let (decoded, _decode_info) = codec.from_str(&json_str, None).await?;

    assert_eq!(decoded, original);

    Ok(())
}

/// Round-trip an article with all direct-mapped inline types
#[tokio::test]
async fn roundtrip_all_inline_types() -> Result<()> {
    let codec = OxaCodec;
    let original = art([p([
        t("plain"),
        em([t("italic")]),
        stg([t("bold")]),
        ci("code"),
        sub([t("sub")]),
        sup([t("sup")]),
    ])]);

    let (json_str, _encode_info) = codec.to_string(&original, None).await?;
    let (decoded, _decode_info) = codec.from_str(&json_str, None).await?;

    assert_eq!(decoded, original);

    Ok(())
}

/// Round-trip an article with all direct-mapped block types
#[tokio::test]
async fn roundtrip_all_block_types() -> Result<()> {
    let codec = OxaCodec;
    let original = art([
        h(1, [t("Heading 1")]),
        h(2, [t("Heading 2")]),
        p([t("A paragraph")]),
        cb("code here", Some("python")),
        tb(),
    ]);

    let (json_str, _encode_info) = codec.to_string(&original, None).await?;
    let (decoded, _decode_info) = codec.from_str(&json_str, None).await?;

    assert_eq!(decoded, original);

    Ok(())
}

/// Round-trip nested formatting: emphasis inside strong inside paragraph
#[tokio::test]
async fn roundtrip_nested_formatting() -> Result<()> {
    let codec = OxaCodec;
    let original = art([p([stg([t("bold and "), em([t("bold-italic")])])])]);

    let (json_str, _encode_info) = codec.to_string(&original, None).await?;
    let (decoded, _decode_info) = codec.from_str(&json_str, None).await?;

    assert_eq!(decoded, original);

    Ok(())
}
