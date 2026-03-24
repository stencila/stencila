//! Tests for OXA schema conformance.
//!
//! These tests verify that the codec's JSON output for each of the 11 directly-mapped
//! type pairs can be deserialized into the corresponding `oxa-types` structs without error.
//! This confirms that the codec produces structurally valid OXA JSON per the OXA type
//! definitions in `oxa-types-rs`.

use serde_json::Value;
use stencila_codec::{
    Codec,
    eyre::{OptionExt, Result},
    stencila_schema::{
        Article, Block, Inline, Node,
        shortcuts::{art, cb, ci, em, h, p, stg, sub, sup, t, tb},
    },
};
use stencila_codec_oxa::OxaCodec;

/// Helper: encode a Stencila node to OXA JSON and return the parsed Value
async fn encode_to_value(node: &Node) -> Result<Value> {
    let codec = OxaCodec;
    let (json_str, _info) = codec.to_string(node, None).await?;
    let value: Value = serde_json::from_str(&json_str)?;
    Ok(value)
}

// ---------------------------------------------------------------------------
// Full document conformance: Article → oxa_types::Document
// ---------------------------------------------------------------------------

/// The full encoded Document JSON deserializes into oxa_types::Document
#[tokio::test]
async fn article_to_oxa_document() -> Result<()> {
    let doc = art([p([t("Hello world")])]);
    let value = encode_to_value(&doc).await?;

    let oxa_doc: oxa_types::Document = serde_json::from_value(value)?;

    assert_eq!(oxa_doc.children.len(), 1);

    Ok(())
}

/// A Document with title and metadata deserializes into oxa_types::Document
#[tokio::test]
async fn article_with_title_to_oxa_document() -> Result<()> {
    let doc = Node::Article(Article {
        title: Some(vec![Inline::Text(
            stencila_codec::stencila_schema::Text::new("My Title".into()),
        )]),
        content: vec![Block::Paragraph(
            stencila_codec::stencila_schema::Paragraph::new(vec![Inline::Text(
                stencila_codec::stencila_schema::Text::new("Body".into()),
            )]),
        )],
        ..Default::default()
    });

    let value = encode_to_value(&doc).await?;
    let oxa_doc: oxa_types::Document = serde_json::from_value(value)?;

    assert!(oxa_doc.title.is_some(), "OXA Document should have a title");
    assert_eq!(oxa_doc.children.len(), 1);

    Ok(())
}

// ---------------------------------------------------------------------------
// Block-level conformance: individual block children
// ---------------------------------------------------------------------------

/// Encoded Paragraph JSON deserializes into oxa_types::Paragraph
#[tokio::test]
async fn paragraph_to_oxa_paragraph() -> Result<()> {
    let doc = art([p([t("Some text")])]);
    let value = encode_to_value(&doc).await?;

    let block_value = value["children"]
        .as_array()
        .ok_or_eyre("missing children")?
        .first()
        .ok_or_eyre("empty children")?
        .clone();

    let oxa_para: oxa_types::Paragraph = serde_json::from_value(block_value)?;
    assert_eq!(oxa_para.children.len(), 1);

    Ok(())
}

/// Encoded Heading JSON deserializes into oxa_types::Heading
#[tokio::test]
async fn heading_to_oxa_heading() -> Result<()> {
    let doc = art([h(2, [t("Section")])]);
    let value = encode_to_value(&doc).await?;

    let block_value = value["children"]
        .as_array()
        .ok_or_eyre("missing children")?
        .first()
        .ok_or_eyre("empty children")?
        .clone();

    let oxa_heading: oxa_types::Heading = serde_json::from_value(block_value)?;
    assert_eq!(oxa_heading.level, 2);
    assert_eq!(oxa_heading.children.len(), 1);

    Ok(())
}

/// Encoded CodeBlock JSON deserializes into oxa_types::Code
#[tokio::test]
async fn code_block_to_oxa_code() -> Result<()> {
    let doc = art([cb("print('hello')", Some("python"))]);
    let value = encode_to_value(&doc).await?;

    let block_value = value["children"]
        .as_array()
        .ok_or_eyre("missing children")?
        .first()
        .ok_or_eyre("empty children")?
        .clone();

    let oxa_code: oxa_types::Code = serde_json::from_value(block_value)?;
    assert_eq!(oxa_code.value, "print('hello')");
    assert_eq!(oxa_code.language.as_deref(), Some("python"));

    Ok(())
}

/// Encoded CodeBlock without language deserializes into oxa_types::Code
#[tokio::test]
async fn code_block_no_language_to_oxa_code() -> Result<()> {
    let doc = art([cb("x = 1", None::<&str>)]);
    let value = encode_to_value(&doc).await?;

    let block_value = value["children"]
        .as_array()
        .ok_or_eyre("missing children")?
        .first()
        .ok_or_eyre("empty children")?
        .clone();

    let oxa_code: oxa_types::Code = serde_json::from_value(block_value)?;
    assert_eq!(oxa_code.value, "x = 1");
    assert!(oxa_code.language.is_none());

    Ok(())
}

/// Encoded ThematicBreak JSON deserializes into oxa_types::ThematicBreak
#[tokio::test]
async fn thematic_break_to_oxa_thematic_break() -> Result<()> {
    let doc = art([tb()]);
    let value = encode_to_value(&doc).await?;

    let block_value = value["children"]
        .as_array()
        .ok_or_eyre("missing children")?
        .first()
        .ok_or_eyre("empty children")?
        .clone();

    let _oxa_tb: oxa_types::ThematicBreak = serde_json::from_value(block_value)?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Inline-level conformance: individual inline children within a paragraph
// ---------------------------------------------------------------------------

/// Helper: encode an article with a paragraph containing the given inlines,
/// then extract the first inline JSON value from the paragraph's children.
async fn encode_inline_to_value(inlines: Vec<Inline>) -> Result<Value> {
    let doc = Node::Article(Article {
        content: vec![Block::Paragraph(
            stencila_codec::stencila_schema::Paragraph::new(inlines),
        )],
        ..Default::default()
    });
    let value = encode_to_value(&doc).await?;

    let inline_value = value["children"]
        .as_array()
        .ok_or_eyre("missing children")?
        .first()
        .ok_or_eyre("empty children")?["children"]
        .as_array()
        .ok_or_eyre("missing inline children")?
        .first()
        .ok_or_eyre("empty inline children")?
        .clone();

    Ok(inline_value)
}

/// Encoded Text JSON deserializes into oxa_types::Text
#[tokio::test]
async fn text_to_oxa_text() -> Result<()> {
    let inline_value = encode_inline_to_value(vec![Inline::Text(
        stencila_codec::stencila_schema::Text::new("hello".into()),
    )])
    .await?;

    let oxa_text: oxa_types::Text = serde_json::from_value(inline_value)?;
    assert_eq!(oxa_text.value, "hello");

    Ok(())
}

/// Encoded Emphasis JSON deserializes into oxa_types::Emphasis
#[tokio::test]
async fn emphasis_to_oxa_emphasis() -> Result<()> {
    let doc = art([p([em([t("italic")])])]);
    let value = encode_to_value(&doc).await?;

    let inline_value = value["children"][0]["children"]
        .as_array()
        .ok_or_eyre("missing inline children")?
        .first()
        .ok_or_eyre("empty inline children")?
        .clone();

    let oxa_em: oxa_types::Emphasis = serde_json::from_value(inline_value)?;
    assert_eq!(oxa_em.children.len(), 1);

    Ok(())
}

/// Encoded Strong JSON deserializes into oxa_types::Strong
#[tokio::test]
async fn strong_to_oxa_strong() -> Result<()> {
    let doc = art([p([stg([t("bold")])])]);
    let value = encode_to_value(&doc).await?;

    let inline_value = value["children"][0]["children"]
        .as_array()
        .ok_or_eyre("missing inline children")?
        .first()
        .ok_or_eyre("empty inline children")?
        .clone();

    let oxa_strong: oxa_types::Strong = serde_json::from_value(inline_value)?;
    assert_eq!(oxa_strong.children.len(), 1);

    Ok(())
}

/// Encoded CodeInline JSON deserializes into oxa_types::InlineCode
#[tokio::test]
async fn code_inline_to_oxa_inline_code() -> Result<()> {
    let doc = art([p([ci("x + 1")])]);
    let value = encode_to_value(&doc).await?;

    let inline_value = value["children"][0]["children"]
        .as_array()
        .ok_or_eyre("missing inline children")?
        .first()
        .ok_or_eyre("empty inline children")?
        .clone();

    let oxa_ic: oxa_types::InlineCode = serde_json::from_value(inline_value)?;
    assert_eq!(oxa_ic.value, "x + 1");

    Ok(())
}

/// Encoded Subscript JSON deserializes into oxa_types::Subscript
#[tokio::test]
async fn subscript_to_oxa_subscript() -> Result<()> {
    let doc = art([p([sub([t("2")])])]);
    let value = encode_to_value(&doc).await?;

    let inline_value = value["children"][0]["children"]
        .as_array()
        .ok_or_eyre("missing inline children")?
        .first()
        .ok_or_eyre("empty inline children")?
        .clone();

    let oxa_sub: oxa_types::Subscript = serde_json::from_value(inline_value)?;
    assert_eq!(oxa_sub.children.len(), 1);

    Ok(())
}

/// Encoded Superscript JSON deserializes into oxa_types::Superscript
#[tokio::test]
async fn superscript_to_oxa_superscript() -> Result<()> {
    let doc = art([p([sup([t("n")])])]);
    let value = encode_to_value(&doc).await?;

    let inline_value = value["children"][0]["children"]
        .as_array()
        .ok_or_eyre("missing inline children")?
        .first()
        .ok_or_eyre("empty inline children")?
        .clone();

    let oxa_sup: oxa_types::Superscript = serde_json::from_value(inline_value)?;
    assert_eq!(oxa_sup.children.len(), 1);

    Ok(())
}

// ---------------------------------------------------------------------------
// Composite conformance: full document with all direct-mapped types
// ---------------------------------------------------------------------------

/// A complex document with all 11 direct-mapped types deserializes into
/// oxa_types::Document and each child can be individually verified
#[tokio::test]
async fn all_direct_types_oxa_conformance() -> Result<()> {
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

    let value = encode_to_value(&doc).await?;

    // The entire document should deserialize
    let oxa_doc: oxa_types::Document = serde_json::from_value(value.clone())?;
    assert_eq!(oxa_doc.children.len(), 4);

    // Verify each block child is deserializable as the expected OXA Block variant
    let children = value["children"]
        .as_array()
        .ok_or_eyre("missing children")?;

    let _: oxa_types::Heading = serde_json::from_value(children[0].clone())?;
    let _: oxa_types::Paragraph = serde_json::from_value(children[1].clone())?;
    let _: oxa_types::Code = serde_json::from_value(children[2].clone())?;
    let _: oxa_types::ThematicBreak = serde_json::from_value(children[3].clone())?;

    // Verify the paragraph's inline children are also conformant
    let para_children = children[1]["children"]
        .as_array()
        .ok_or_eyre("missing paragraph children")?;

    // Check that each inline child can be deserialized via the oxa_types::Inline enum
    for child in para_children {
        let _: oxa_types::Inline = serde_json::from_value(child.clone())?;
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Block enum conformance: oxa_types::Block deserialization
// ---------------------------------------------------------------------------

/// Each block child deserializes via oxa_types::Block enum (untagged)
#[tokio::test]
async fn blocks_deserialize_as_oxa_block_enum() -> Result<()> {
    let doc = art([
        p([t("para")]),
        h(3, [t("heading")]),
        cb("code", Some("python")),
        tb(),
    ]);

    let value = encode_to_value(&doc).await?;
    let children = value["children"]
        .as_array()
        .ok_or_eyre("missing children")?;

    for child in children {
        let _: oxa_types::Block = serde_json::from_value(child.clone())?;
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Inline enum conformance: oxa_types::Inline deserialization
// ---------------------------------------------------------------------------

/// Each inline child deserializes via oxa_types::Inline enum (untagged)
#[tokio::test]
async fn inlines_deserialize_as_oxa_inline_enum() -> Result<()> {
    let doc = art([p([
        t("text"),
        em([t("em")]),
        stg([t("str")]),
        ci("code"),
        sub([t("sub")]),
        sup([t("sup")]),
    ])]);

    let value = encode_to_value(&doc).await?;
    let inlines = value["children"][0]["children"]
        .as_array()
        .ok_or_eyre("missing inline children")?;

    for inline in inlines {
        let _: oxa_types::Inline = serde_json::from_value(inline.clone())?;
    }

    Ok(())
}
