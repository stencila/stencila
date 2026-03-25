//! Tests for generic fallback decoding and loss tracking (Phase 3 / Slice 1).
//!
//! These tests verify:
//! - Unknown OXA block types decode to `RawBlock` with format
//!   `"application/vnd.oxa+json"` and verbatim JSON content
//! - Unknown OXA inline types decode to `Text` with recursive text extraction
//!   and a loss recorded
//! - OXA `classes` values are dropped on decode with a
//!   `decode:oxa_classes_dropped` loss entry
//! - All loss categories from the design are populated where applicable:
//!   `encode:generic`, `decode:unknown_block_to_raw`,
//!   `decode:unknown_inline_to_text`, `decode:oxa_classes_dropped`

use pretty_assertions::assert_eq;
use serde_json::Value;
use stencila_codec::{
    Codec,
    eyre::{self, Result},
    stencila_schema::{
        Block, Inline, Node,
        shortcuts::{art, p, t},
    },
};
use stencila_codec_oxa::OxaCodec;

// ---------------------------------------------------------------------------
// Helper: serialize Losses to a serde_json::Value for key inspection
// ---------------------------------------------------------------------------

fn losses_to_value(losses: &stencila_codec::Losses) -> Value {
    serde_json::to_value(losses).expect("Losses should always be serializable to JSON")
}

fn losses_contains(losses: &stencila_codec::Losses, key: &str) -> bool {
    let v = losses_to_value(losses);
    v.as_object().is_some_and(|obj| obj.contains_key(key))
}

// ===========================================================================
// Unknown block type → RawBlock
// ===========================================================================

/// An unknown OXA block type (e.g. "Blockquote") should decode to a RawBlock
/// with format "application/vnd.oxa+json" and verbatim JSON as content.
#[tokio::test]
async fn decode_unknown_block_to_raw_block() -> Result<()> {
    let codec = OxaCodec;
    let oxa_json = r#"{
        "type": "Document",
        "children": [
            {
                "type": "Blockquote",
                "children": [
                    {
                        "type": "Paragraph",
                        "children": [{"type": "Text", "value": "quoted text"}]
                    }
                ]
            }
        ]
    }"#;

    let (node, _info) = codec.from_str(oxa_json, None).await?;

    let Node::Article(article) = &node else {
        eyre::bail!("Expected Node::Article");
    };

    assert_eq!(article.content.len(), 1, "Should have exactly one block");

    let Block::RawBlock(raw) = &article.content[0] else {
        panic!(
            "Expected Block::RawBlock for unknown type, got: {:?}",
            article.content[0]
        );
    };

    assert_eq!(
        raw.format, "application/vnd.oxa+json",
        "RawBlock format should be application/vnd.oxa+json"
    );

    // The content should be valid JSON containing the original object
    let content_value: Value = serde_json::from_str(raw.content.as_str())?;
    assert_eq!(
        content_value["type"], "Blockquote",
        "Verbatim JSON should preserve the original type"
    );
    assert!(
        content_value["children"].is_array(),
        "Verbatim JSON should preserve the children array"
    );

    Ok(())
}

/// Multiple unknown block types should each decode to their own RawBlock
#[tokio::test]
async fn decode_multiple_unknown_blocks() -> Result<()> {
    let codec = OxaCodec;
    let oxa_json = r#"{
        "type": "Document",
        "children": [
            {"type": "Aside", "data": {"note": "side content"}},
            {"type": "Paragraph", "children": [{"type": "Text", "value": "normal"}]},
            {"type": "Callout", "kind": "warning", "children": []}
        ]
    }"#;

    let (node, _info) = codec.from_str(oxa_json, None).await?;

    let Node::Article(article) = &node else {
        eyre::bail!("Expected Node::Article");
    };

    assert_eq!(article.content.len(), 3);

    // First block: unknown "Aside" → RawBlock with correct format and preserved JSON
    let Block::RawBlock(aside_raw) = &article.content[0] else {
        panic!(
            "Unknown 'Aside' should become RawBlock, got: {:?}",
            article.content[0]
        );
    };
    assert_eq!(
        aside_raw.format, "application/vnd.oxa+json",
        "Aside RawBlock format should be application/vnd.oxa+json"
    );
    let aside_json: Value = serde_json::from_str(aside_raw.content.as_str())?;
    assert_eq!(
        aside_json["type"], "Aside",
        "Aside JSON type should be preserved"
    );

    // Second block: known "Paragraph" → Paragraph
    assert!(
        matches!(&article.content[1], Block::Paragraph(_)),
        "Known 'Paragraph' should remain Paragraph"
    );

    // Third block: unknown "Callout" → RawBlock with correct format and preserved JSON
    let Block::RawBlock(callout_raw) = &article.content[2] else {
        panic!(
            "Unknown 'Callout' should become RawBlock, got: {:?}",
            article.content[2]
        );
    };
    assert_eq!(
        callout_raw.format, "application/vnd.oxa+json",
        "Callout RawBlock format should be application/vnd.oxa+json"
    );
    let callout_json: Value = serde_json::from_str(callout_raw.content.as_str())?;
    assert_eq!(
        callout_json["type"], "Callout",
        "Callout JSON type should be preserved"
    );

    Ok(())
}

/// Unknown block with no children and only data fields should still decode to RawBlock
#[tokio::test]
async fn decode_unknown_block_data_only() -> Result<()> {
    let codec = OxaCodec;
    let oxa_json = r#"{
        "type": "Document",
        "children": [
            {
                "type": "MystDirective",
                "data": {"name": "note", "value": "important info"}
            }
        ]
    }"#;

    let (node, _info) = codec.from_str(oxa_json, None).await?;

    let Node::Article(article) = &node else {
        eyre::bail!("Expected Node::Article");
    };

    let Block::RawBlock(raw) = &article.content[0] else {
        panic!("Expected RawBlock for unknown type 'MystDirective'");
    };

    assert_eq!(raw.format, "application/vnd.oxa+json");
    let content_value: Value = serde_json::from_str(raw.content.as_str())?;
    assert_eq!(content_value["type"], "MystDirective");

    Ok(())
}

// ===========================================================================
// Unknown inline type → Text (with recursive text extraction)
// ===========================================================================

/// An unknown inline type (e.g. "Abbreviation") should decode to a Text node
/// with its text content recursively extracted.
#[tokio::test]
async fn decode_unknown_inline_to_text() -> Result<()> {
    let codec = OxaCodec;
    let oxa_json = r#"{
        "type": "Document",
        "children": [
            {
                "type": "Paragraph",
                "children": [
                    {
                        "type": "Abbreviation",
                        "title": "HyperText Markup Language",
                        "children": [{"type": "Text", "value": "HTML"}]
                    }
                ]
            }
        ]
    }"#;

    let (node, _info) = codec.from_str(oxa_json, None).await?;

    let Node::Article(article) = &node else {
        eyre::bail!("Expected Node::Article");
    };

    let Block::Paragraph(para) = &article.content[0] else {
        panic!("Expected Paragraph");
    };

    assert_eq!(para.content.len(), 1);
    let Inline::Text(text) = &para.content[0] else {
        panic!(
            "Expected Inline::Text for unknown inline type, got: {:?}",
            para.content[0]
        );
    };

    assert_eq!(
        text.value.as_str(),
        "HTML",
        "Text value should be recursively extracted from children"
    );

    Ok(())
}

/// Unknown inline with nested children should recursively extract all text
#[tokio::test]
async fn decode_unknown_inline_recursive_text_extraction() -> Result<()> {
    let codec = OxaCodec;
    let oxa_json = r#"{
        "type": "Document",
        "children": [
            {
                "type": "Paragraph",
                "children": [
                    {
                        "type": "Highlight",
                        "children": [
                            {"type": "Text", "value": "hello "},
                            {"type": "Strong", "children": [{"type": "Text", "value": "world"}]}
                        ]
                    }
                ]
            }
        ]
    }"#;

    let (node, _info) = codec.from_str(oxa_json, None).await?;

    let Node::Article(article) = &node else {
        eyre::bail!("Expected Node::Article");
    };

    let Block::Paragraph(para) = &article.content[0] else {
        panic!("Expected Paragraph");
    };

    assert_eq!(para.content.len(), 1);
    let Inline::Text(text) = &para.content[0] else {
        panic!("Expected Inline::Text for unknown inline type 'Highlight'");
    };

    assert_eq!(
        text.value.as_str(),
        "hello world",
        "Recursive text extraction should concatenate all nested text"
    );

    Ok(())
}

/// Unknown inline with a value field (no children) should use the value as text
#[tokio::test]
async fn decode_unknown_inline_with_value() -> Result<()> {
    let codec = OxaCodec;
    let oxa_json = r#"{
        "type": "Document",
        "children": [
            {
                "type": "Paragraph",
                "children": [
                    {
                        "type": "MathInline",
                        "value": "E = mc^2"
                    }
                ]
            }
        ]
    }"#;

    let (node, _info) = codec.from_str(oxa_json, None).await?;

    let Node::Article(article) = &node else {
        eyre::bail!("Expected Node::Article");
    };

    let Block::Paragraph(para) = &article.content[0] else {
        panic!("Expected Paragraph");
    };

    let Inline::Text(text) = &para.content[0] else {
        panic!("Expected Inline::Text for unknown inline type 'MathInline'");
    };

    assert_eq!(
        text.value.as_str(),
        "E = mc^2",
        "Unknown inline with value field should use value as text"
    );

    Ok(())
}

// ===========================================================================
// OXA classes dropping
// ===========================================================================

/// Classes on a known block type should be dropped (Stencila has no `classes`
/// field on most types) and a decode loss should be recorded.
#[tokio::test]
async fn decode_block_with_classes_dropped() -> Result<()> {
    let codec = OxaCodec;
    let oxa_json = r#"{
        "type": "Document",
        "children": [
            {
                "type": "Paragraph",
                "classes": ["highlight", "important"],
                "children": [{"type": "Text", "value": "styled text"}]
            }
        ]
    }"#;

    let (node, info) = codec.from_str(oxa_json, None).await?;

    let Node::Article(article) = &node else {
        eyre::bail!("Expected Node::Article");
    };

    // The paragraph should still decode successfully
    let Block::Paragraph(para) = &article.content[0] else {
        panic!("Expected Paragraph");
    };

    assert_eq!(para.content.len(), 1);

    // The classes should be silently dropped — verify via losses
    assert!(
        losses_contains(&info.losses, "decode:oxa_classes_dropped"),
        "Losses should contain 'decode:oxa_classes_dropped' entry, got: {:?}",
        losses_to_value(&info.losses)
    );

    Ok(())
}

/// Classes on inline types should also be dropped with a loss recorded
#[tokio::test]
async fn decode_inline_with_classes_dropped() -> Result<()> {
    let codec = OxaCodec;
    let oxa_json = r#"{
        "type": "Document",
        "children": [
            {
                "type": "Paragraph",
                "children": [
                    {
                        "type": "Emphasis",
                        "classes": ["special"],
                        "children": [{"type": "Text", "value": "fancy italic"}]
                    }
                ]
            }
        ]
    }"#;

    let (node, info) = codec.from_str(oxa_json, None).await?;

    let Node::Article(article) = &node else {
        eyre::bail!("Expected Node::Article");
    };

    let Block::Paragraph(para) = &article.content[0] else {
        panic!("Expected Paragraph");
    };

    // The emphasis should still decode correctly
    let Inline::Emphasis(em) = &para.content[0] else {
        panic!("Expected Emphasis inline");
    };
    assert_eq!(em.content.len(), 1);

    // Classes should be recorded as lost
    assert!(
        losses_contains(&info.losses, "decode:oxa_classes_dropped"),
        "Losses should contain 'decode:oxa_classes_dropped' when classes are present on inlines"
    );

    Ok(())
}

// ===========================================================================
// Loss tracking: decode losses
// ===========================================================================

/// Decoding an unknown block should record a decode:unknown_block_to_raw loss
#[tokio::test]
async fn decode_loss_unknown_block_to_raw() -> Result<()> {
    let codec = OxaCodec;
    let oxa_json = r#"{
        "type": "Document",
        "children": [
            {"type": "CustomWidget", "data": {"id": 42}}
        ]
    }"#;

    let (_node, info) = codec.from_str(oxa_json, None).await?;

    assert!(
        losses_contains(&info.losses, "decode:unknown_block_to_raw"),
        "Decoding unknown block should record 'decode:unknown_block_to_raw' loss, got: {:?}",
        losses_to_value(&info.losses)
    );

    Ok(())
}

/// Decoding an unknown inline should record a decode:unknown_inline_to_text loss
#[tokio::test]
async fn decode_loss_unknown_inline_to_text() -> Result<()> {
    let codec = OxaCodec;
    let oxa_json = r#"{
        "type": "Document",
        "children": [
            {
                "type": "Paragraph",
                "children": [
                    {
                        "type": "Footnote",
                        "children": [{"type": "Text", "value": "note"}]
                    }
                ]
            }
        ]
    }"#;

    let (_node, info) = codec.from_str(oxa_json, None).await?;

    assert!(
        losses_contains(&info.losses, "decode:unknown_inline_to_text"),
        "Decoding unknown inline should record 'decode:unknown_inline_to_text' loss, got: {:?}",
        losses_to_value(&info.losses)
    );

    Ok(())
}

/// Decoding a document with classes should record decode:oxa_classes_dropped
#[tokio::test]
async fn decode_loss_classes_dropped() -> Result<()> {
    let codec = OxaCodec;
    let oxa_json = r#"{
        "type": "Document",
        "children": [
            {
                "type": "Heading",
                "level": 1,
                "classes": ["title-class"],
                "children": [{"type": "Text", "value": "Title"}]
            }
        ]
    }"#;

    let (_node, info) = codec.from_str(oxa_json, None).await?;

    assert!(
        losses_contains(&info.losses, "decode:oxa_classes_dropped"),
        "Decoding nodes with classes should record 'decode:oxa_classes_dropped' loss, got: {:?}",
        losses_to_value(&info.losses)
    );

    Ok(())
}

/// A document with only known types and no classes should have no decode losses
#[tokio::test]
async fn decode_no_losses_for_known_types() -> Result<()> {
    let codec = OxaCodec;
    let oxa_json = r#"{
        "type": "Document",
        "children": [
            {
                "type": "Paragraph",
                "children": [{"type": "Text", "value": "hello"}]
            }
        ]
    }"#;

    let (_node, info) = codec.from_str(oxa_json, None).await?;

    assert!(
        info.losses.is_empty(),
        "No losses expected for a document with only known types, got: {:?}",
        losses_to_value(&info.losses)
    );

    Ok(())
}

// ===========================================================================
// Loss tracking: encode losses
// ===========================================================================

/// Encoding a non-directly-mapped block type should record an encode:generic loss
#[tokio::test]
async fn encode_loss_generic_block() -> Result<()> {
    let codec = OxaCodec;
    // A List is not directly mapped — it uses the generic fallback
    let doc = art([Block::List(stencila_codec::stencila_schema::List {
        items: vec![stencila_codec::stencila_schema::ListItem {
            content: vec![p([t("item")])],
            ..Default::default()
        }],
        ..Default::default()
    })]);

    let (_json_str, info) = codec.to_string(&doc, None).await?;

    assert!(
        losses_contains(&info.losses, "encode:generic"),
        "Encoding non-directly-mapped block should record 'encode:generic' loss, got: {:?}",
        losses_to_value(&info.losses)
    );

    Ok(())
}

/// Encoding a document with only direct-mapped types should have no losses
#[tokio::test]
async fn encode_no_losses_for_direct_types() -> Result<()> {
    let codec = OxaCodec;
    let doc = art([p([t("hello")])]);

    let (_json_str, info) = codec.to_string(&doc, None).await?;

    assert!(
        info.losses.is_empty(),
        "No losses expected for directly-mapped types, got: {:?}",
        losses_to_value(&info.losses)
    );

    Ok(())
}

// ===========================================================================
// Combined: a complex document exercises all loss categories at once
// ===========================================================================

/// A document with unknown blocks, unknown inlines, and classes exercises
/// multiple loss categories in a single decode.
#[tokio::test]
async fn decode_combined_losses() -> Result<()> {
    let codec = OxaCodec;
    let oxa_json = r#"{
        "type": "Document",
        "children": [
            {
                "type": "Blockquote",
                "children": [
                    {"type": "Paragraph", "children": [{"type": "Text", "value": "quote"}]}
                ]
            },
            {
                "type": "Paragraph",
                "classes": ["intro"],
                "children": [
                    {"type": "Text", "value": "Before "},
                    {
                        "type": "Abbreviation",
                        "children": [{"type": "Text", "value": "HTML"}]
                    },
                    {"type": "Text", "value": " after"}
                ]
            }
        ]
    }"#;

    let (_node, info) = codec.from_str(oxa_json, None).await?;

    let loss_value = losses_to_value(&info.losses);

    assert!(
        losses_contains(&info.losses, "decode:unknown_block_to_raw"),
        "Should have unknown_block_to_raw loss, got: {loss_value:?}"
    );
    assert!(
        losses_contains(&info.losses, "decode:unknown_inline_to_text"),
        "Should have unknown_inline_to_text loss, got: {loss_value:?}"
    );
    assert!(
        losses_contains(&info.losses, "decode:oxa_classes_dropped"),
        "Should have oxa_classes_dropped loss, got: {loss_value:?}"
    );

    Ok(())
}
