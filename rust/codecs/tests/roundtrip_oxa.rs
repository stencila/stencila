//! CLI-level integration tests for `stencila convert` round-trip with OXA format.
//!
//! These tests exercise the same code path as the `stencila convert` CLI command
//! by using `stencila_codecs::convert`, `from_path`, and `to_path` with temporary
//! files that have `.oxa.json` extensions. This verifies that:
//!
//! - The OXA codec is properly registered and dispatched via file extension
//! - An Article with direct-mapped types survives a JSON → OXA → JSON round-trip
//! - An Article with generic-fallback types survives an OXA encode → OXA decode
//!   round-trip to the extent permitted by the fallback (unknown blocks become
//!   RawBlock, etc.)
//! - Loss information is propagated through the codecs-level functions

use pretty_assertions::assert_eq;
use stencila_codec::{
    DecodeOptions, EncodeOptions,
    eyre::{self, Result},
    stencila_format::Format,
    stencila_schema::{
        Block, Node,
        shortcuts::{art, cb, em, h, p, stg, t, tb},
    },
};

// ===========================================================================
// File-based round-trip via stencila_codecs::convert (same path as CLI)
// ===========================================================================

/// Round-trip a simple article through file-based convert:
/// write JSON → convert to .oxa.json → convert back to JSON → compare nodes.
///
/// This exercises the exact same dispatch path as `stencila convert input.json output.oxa.json`.
#[tokio::test]
async fn convert_roundtrip_simple_article_via_files() -> Result<()> {
    let original = art([
        h(1, [t("Title")]),
        p([t("A paragraph with "), em([t("emphasis")]), t(".")]),
        cb("print('hello')", Some("python")),
        tb(),
    ]);

    // Encode original to JSON (source format)
    let json_str = stencila_codecs::to_string(
        &original,
        Some(EncodeOptions {
            format: Some(Format::Json),
            compact: Some(false),
            ..Default::default()
        }),
    )
    .await?;

    let dir = tempfile::tempdir()?;
    let json_input = dir.path().join("input.json");
    let oxa_file = dir.path().join("intermediate.oxa");
    let json_output = dir.path().join("output.json");

    tokio::fs::write(&json_input, &json_str).await?;

    // Convert JSON → OXA (like: stencila convert input.json intermediate.oxa)
    stencila_codecs::convert(
        Some(json_input.as_path()),
        Some(oxa_file.as_path()),
        None,
        None,
    )
    .await?;

    // Verify the OXA file was created and contains valid JSON with type "Document"
    let oxa_content = tokio::fs::read_to_string(&oxa_file).await?;
    let oxa_value: serde_json::Value = serde_json::from_str(&oxa_content)?;
    assert_eq!(
        oxa_value["type"], "Document",
        "OXA output should have type 'Document'"
    );

    // Convert OXA → JSON (like: stencila convert intermediate.oxa output.json)
    stencila_codecs::convert(
        Some(oxa_file.as_path()),
        Some(json_output.as_path()),
        None,
        None,
    )
    .await?;

    // Read back the output and decode
    let round_tripped = stencila_codecs::from_path(
        json_output.as_path(),
        Some(DecodeOptions {
            format: Some(Format::Json),
            ..Default::default()
        }),
    )
    .await?;

    assert_eq!(
        round_tripped, original,
        "Article with direct-mapped types should survive JSON → OXA → JSON round-trip"
    );

    Ok(())
}

/// Round-trip all direct-mapped inline types through file-based convert.
#[tokio::test]
async fn convert_roundtrip_all_inline_types_via_files() -> Result<()> {
    use stencila_codec::stencila_schema::shortcuts::{ci, sub, sup};

    let original = art([p([
        t("plain "),
        em([t("italic ")]),
        stg([t("bold ")]),
        ci("code"),
        t(" "),
        sub([t("sub")]),
        t(" "),
        sup([t("sup")]),
    ])]);

    let json_str = stencila_codecs::to_string(
        &original,
        Some(EncodeOptions {
            format: Some(Format::Json),
            compact: Some(false),
            ..Default::default()
        }),
    )
    .await?;

    let dir = tempfile::tempdir()?;
    let json_input = dir.path().join("input.json");
    let oxa_file = dir.path().join("intermediate.oxa");
    let json_output = dir.path().join("output.json");

    tokio::fs::write(&json_input, &json_str).await?;

    stencila_codecs::convert(
        Some(json_input.as_path()),
        Some(oxa_file.as_path()),
        None,
        None,
    )
    .await?;

    stencila_codecs::convert(
        Some(oxa_file.as_path()),
        Some(json_output.as_path()),
        None,
        None,
    )
    .await?;

    let round_tripped = stencila_codecs::from_path(
        json_output.as_path(),
        Some(DecodeOptions {
            format: Some(Format::Json),
            ..Default::default()
        }),
    )
    .await?;

    assert_eq!(
        round_tripped, original,
        "All direct-mapped inline types should survive round-trip via file convert"
    );

    Ok(())
}

// ===========================================================================
// String-based round-trip via codecs-level dispatch (format-based)
// ===========================================================================

/// Round-trip a simple article through string-based codecs dispatch:
/// encode to OXA string → decode from OXA string → compare.
///
/// This tests the codecs-level dispatch with Format::Oxa, which is
/// the same dispatch that `stencila convert --from oxa --to oxa` uses.
#[tokio::test]
async fn convert_roundtrip_via_codecs_dispatch() -> Result<()> {
    let original = art([
        h(2, [t("Section")]),
        p([t("Normal "), stg([t("bold")]), t(".")]),
        tb(),
    ]);

    // Encode via codecs dispatch (format = Oxa)
    let oxa_str = stencila_codecs::to_string(
        &original,
        Some(EncodeOptions {
            format: Some(Format::Oxa),
            ..Default::default()
        }),
    )
    .await?;

    // Decode via codecs dispatch (format = Oxa)
    let decoded = stencila_codecs::from_str(
        &oxa_str,
        Some(DecodeOptions {
            format: Some(Format::Oxa),
            ..Default::default()
        }),
    )
    .await?;

    assert_eq!(
        decoded, original,
        "Article should survive OXA encode → decode round-trip via codecs dispatch"
    );

    Ok(())
}

// ===========================================================================
// Round-trip with generic-fallback types: encode loses fidelity, decode
// produces RawBlock for unknown types
// ===========================================================================

/// When an article with non-directly-mapped types (e.g. List) is encoded to OXA
/// and decoded back, the generic-fallback types become RawBlock on decode.
/// This test verifies the round-trip still succeeds (no errors) and that
/// directly-mapped types within the same document survive intact.
#[tokio::test]
async fn convert_roundtrip_mixed_direct_and_generic_types() -> Result<()> {
    let original = art([
        h(1, [t("Title")]),
        // List is not directly mapped — goes through generic fallback
        Block::List(stencila_codec::stencila_schema::List {
            items: vec![stencila_codec::stencila_schema::ListItem {
                content: vec![p([t("item one")])],
                ..Default::default()
            }],
            ..Default::default()
        }),
        p([t("After the list")]),
    ]);

    let oxa_str = stencila_codecs::to_string(
        &original,
        Some(EncodeOptions {
            format: Some(Format::Oxa),
            ..Default::default()
        }),
    )
    .await?;

    let decoded = stencila_codecs::from_str(
        &oxa_str,
        Some(DecodeOptions {
            format: Some(Format::Oxa),
            ..Default::default()
        }),
    )
    .await?;

    let Node::Article(article) = &decoded else {
        eyre::bail!("Expected Node::Article");
    };

    // First block: Heading should survive intact
    assert!(
        matches!(&article.content[0], Block::Heading(_)),
        "Direct-mapped Heading should survive round-trip"
    );

    // Second block: List was generically encoded → on decode should be RawBlock
    // (because the generic encoding produces a type like "List" which is unknown to decode)
    assert!(
        matches!(&article.content[1], Block::RawBlock(_)),
        "Generic-encoded List should decode to RawBlock, got: {:?}",
        article.content[1]
    );

    // Third block: Paragraph should survive intact
    assert!(
        matches!(&article.content[2], Block::Paragraph(_)),
        "Direct-mapped Paragraph should survive round-trip"
    );

    Ok(())
}

// ===========================================================================
// Loss propagation through codecs-level functions
// ===========================================================================

/// Encoding a document with generic-fallback types through the codecs-level
/// dispatch should propagate encode:generic losses.
#[tokio::test]
async fn convert_encode_losses_propagated() -> Result<()> {
    let doc = art([Block::List(stencila_codec::stencila_schema::List {
        items: vec![stencila_codec::stencila_schema::ListItem {
            content: vec![p([t("item")])],
            ..Default::default()
        }],
        ..Default::default()
    })]);

    let (_oxa_str, encode_info) = stencila_codecs::to_string_with_info(
        &doc,
        Some(EncodeOptions {
            format: Some(Format::Oxa),
            ..Default::default()
        }),
    )
    .await?;

    let loss_json =
        serde_json::to_value(&encode_info.losses).expect("Losses should be serializable");
    assert!(
        loss_json
            .as_object()
            .is_some_and(|obj| obj.contains_key("generic")),
        "Encode losses should contain 'generic' when using generic fallback, got: {loss_json:?}"
    );

    Ok(())
}

/// Decoding OXA JSON with unknown types through the codecs-level dispatch
/// should propagate decode losses.
#[tokio::test]
async fn convert_decode_losses_propagated() -> Result<()> {
    let oxa_json = r#"{
        "type": "Document",
        "children": [
            {
                "type": "Blockquote",
                "children": [
                    {"type": "Paragraph", "children": [{"type": "Text", "value": "quoted"}]}
                ]
            },
            {
                "type": "Paragraph",
                "classes": ["intro"],
                "children": [
                    {"type": "Text", "value": "Before "},
                    {"type": "Abbreviation", "children": [{"type": "Text", "value": "HTML"}]}
                ]
            }
        ]
    }"#;

    let (_node, decode_info) = stencila_codecs::from_str_with_info(
        oxa_json,
        Some(DecodeOptions {
            format: Some(Format::Oxa),
            ..Default::default()
        }),
    )
    .await?;

    let loss_json =
        serde_json::to_value(&decode_info.losses).expect("Losses should be serializable");
    let loss_obj = loss_json
        .as_object()
        .expect("Losses should serialize to a JSON object");

    assert!(
        loss_obj.contains_key("unknown_block_to_raw"),
        "Decode losses should contain 'unknown_block_to_raw', got: {loss_json:?}"
    );
    assert!(
        loss_obj.contains_key("unknown_inline_to_text"),
        "Decode losses should contain 'unknown_inline_to_text', got: {loss_json:?}"
    );
    assert!(
        loss_obj.contains_key("oxa_classes_dropped"),
        "Decode losses should contain 'oxa_classes_dropped', got: {loss_json:?}"
    );

    Ok(())
}
