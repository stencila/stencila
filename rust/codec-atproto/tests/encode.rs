//! Tests for the top-level article encoding function in `encode.rs` and
//! the `Codec` trait implementation.
//!
//! Each test covers one or more acceptance criteria from Phase 2 / Slice 2.

use pretty_assertions::assert_eq;
use serde_json::Value;
use stencila_codec::{
    Codec, CodecSupport, Losses,
    stencila_format::Format,
    stencila_schema::{
        Article, ArticleOptions, Author, DateTime, Node, Person,
        shortcuts::{art, em, p, t},
    },
};
use stencila_codec_atproto::{
    AtProtoCodec,
    nsids::{OXA_EMPHASIS, OXA_PARAGRAPH},
};

// ---------------------------------------------------------------------------
// Helper: serialize Losses to a serde_json::Value for key inspection
// ---------------------------------------------------------------------------

fn losses_to_value(losses: &Losses) -> Value {
    serde_json::to_value(losses).expect("Losses should always be serializable to JSON")
}

fn losses_contains(losses: &Losses, key: &str) -> bool {
    let v = losses_to_value(losses);
    v.as_object().is_some_and(|obj| obj.contains_key(key))
}

// ===========================================================================
// AC1: Complete article with title, content blocks, and date_published
//      encodes to correct top-level JSON structure
// ===========================================================================

#[tokio::test]
async fn article_with_title_content_and_date_published_encodes_correctly() {
    let doc = Node::Article(Article {
        title: Some(vec![t("My Title")]),
        date_published: Some(DateTime::new("2024-06-15".into())),
        content: vec![p([t("Hello world")])],
        ..Default::default()
    });

    let codec = AtProtoCodec;
    let (json_str, _info) = codec
        .to_string(&doc, None)
        .await
        .expect("Encoding should succeed");
    let value: Value = serde_json::from_str(&json_str).expect("Output should be valid JSON");

    // Should have a blocks array with one paragraph block
    let blocks = value["blocks"]
        .as_array()
        .expect("Output should have a blocks array");
    assert_eq!(
        blocks.len(),
        1,
        "blocks array should have exactly one block"
    );

    // First block should be a paragraph with the correct $type and text content
    let first_block = &blocks[0];
    assert_eq!(
        first_block["$type"].as_str(),
        Some(OXA_PARAGRAPH),
        "First block should have the paragraph $type"
    );
    assert_eq!(
        first_block["text"].as_str(),
        Some("Hello world"),
        "First block text should be 'Hello world'"
    );

    // Title should be an object with text and no facets (plain text title)
    let title = &value["title"];
    assert_eq!(
        title["text"].as_str(),
        Some("My Title"),
        "Title text should be 'My Title'"
    );
    assert!(
        title.get("facets").is_none() || title["facets"].as_array().is_some_and(|a| a.is_empty()),
        "Plain-text title should have no facets or an empty facets array"
    );

    // createdAt should come from date_published
    let created_at = value["createdAt"]
        .as_str()
        .expect("createdAt should be a string");
    assert!(
        created_at.starts_with("2024-06-15"),
        "createdAt should start with the date_published date, got: {created_at}"
    );
}

// ===========================================================================
// AC2: Article with only date_created uses date_created for createdAt
// ===========================================================================

#[tokio::test]
async fn article_date_created_used_when_no_date_published() {
    let doc = Node::Article(Article {
        content: vec![p([t("Content")])],
        options: Box::new(ArticleOptions {
            date_created: Some(DateTime::new("2023-01-10".into())),
            ..Default::default()
        }),
        ..Default::default()
    });

    let codec = AtProtoCodec;
    let (json_str, _info) = codec
        .to_string(&doc, None)
        .await
        .expect("Encoding should succeed");
    let value: Value = serde_json::from_str(&json_str).expect("Output should be valid JSON");

    let created_at = value["createdAt"]
        .as_str()
        .expect("createdAt should be a string");
    assert!(
        created_at.starts_with("2023-01-10"),
        "createdAt should use date_created when date_published is absent, got: {created_at}"
    );
}

// ===========================================================================
// AC3: Article with only date_modified uses date_modified for createdAt
// ===========================================================================

#[tokio::test]
async fn article_date_modified_used_when_no_other_dates() {
    let doc = Node::Article(Article {
        content: vec![p([t("Content")])],
        options: Box::new(ArticleOptions {
            date_modified: Some(DateTime::new("2025-03-20".into())),
            ..Default::default()
        }),
        ..Default::default()
    });

    let codec = AtProtoCodec;
    let (json_str, _info) = codec
        .to_string(&doc, None)
        .await
        .expect("Encoding should succeed");
    let value: Value = serde_json::from_str(&json_str).expect("Output should be valid JSON");

    let created_at = value["createdAt"]
        .as_str()
        .expect("createdAt should be a string");
    assert!(
        created_at.starts_with("2025-03-20"),
        "createdAt should use date_modified when other dates are absent, got: {created_at}"
    );
}

// ===========================================================================
// AC4: Article without any date uses current time for createdAt in ISO 8601
// ===========================================================================

#[tokio::test]
async fn article_without_dates_uses_current_time() {
    let before = chrono::Utc::now();

    let doc = art([p([t("No dates")])]);

    let codec = AtProtoCodec;
    let (json_str, _info) = codec
        .to_string(&doc, None)
        .await
        .expect("Encoding should succeed");
    let value: Value = serde_json::from_str(&json_str).expect("Output should be valid JSON");

    let after = chrono::Utc::now();

    let created_at = value["createdAt"]
        .as_str()
        .expect("createdAt should be a string");

    // Should be parseable as a valid ISO 8601 / RFC 3339 datetime
    let parsed = chrono::DateTime::parse_from_rfc3339(created_at).unwrap_or_else(|e| {
        panic!("createdAt should be valid RFC 3339 / ISO 8601, got '{created_at}': {e}")
    });
    let parsed_utc = parsed.with_timezone(&chrono::Utc);

    // The generated timestamp should be between `before` and `after`
    assert!(
        parsed_utc >= before && parsed_utc <= after,
        "createdAt should be near the current time; before={before}, got={parsed_utc}, after={after}"
    );
}

// ===========================================================================
// AC5: Article with authors and abstract records dropped-property losses
// ===========================================================================

#[tokio::test]
async fn article_with_authors_and_abstract_records_losses() {
    let doc = Node::Article(Article {
        content: vec![p([t("Body")])],
        authors: Some(vec![Author::Person(Person {
            family_names: Some(vec!["Doe".into()]),
            ..Default::default()
        })]),
        r#abstract: Some(vec![p([t("This is the abstract.")])]),
        ..Default::default()
    });

    let codec = AtProtoCodec;
    let (_json_str, info) = codec
        .to_string(&doc, None)
        .await
        .expect("Encoding should succeed");

    assert!(
        losses_contains(&info.losses, "encode:dropped_article_authors"),
        "Should record encode:dropped_article_authors loss, got: {:?}",
        losses_to_value(&info.losses)
    );
    assert!(
        losses_contains(&info.losses, "encode:dropped_article_abstract"),
        "Should record encode:dropped_article_abstract loss, got: {:?}",
        losses_to_value(&info.losses)
    );
}

// ===========================================================================
// AC6: Article without title produces no title field in output
// ===========================================================================

#[tokio::test]
async fn article_without_title_has_no_title_field() {
    let doc = art([p([t("No title article")])]);

    let codec = AtProtoCodec;
    let (json_str, _info) = codec
        .to_string(&doc, None)
        .await
        .expect("Encoding should succeed");
    let value: Value = serde_json::from_str(&json_str).expect("Output should be valid JSON");

    assert!(
        value.get("title").is_none() || value["title"].is_null(),
        "Article without title should not have a title field in output"
    );
}

// ===========================================================================
// AC7: Non-Article node returns an error
// ===========================================================================

#[tokio::test]
async fn non_article_node_returns_error() {
    // A Paragraph wrapped directly as a Node is not an Article
    let non_article = Node::Paragraph(stencila_codec::stencila_schema::Paragraph {
        content: vec![t("Not an article")],
        ..Default::default()
    });

    let codec = AtProtoCodec;
    let result = codec.to_string(&non_article, None).await;

    assert!(
        result.is_err(),
        "Encoding a non-Article node should return an error"
    );
}

// ===========================================================================
// AC1 supplement: Title with formatted inlines includes facets with feature type
// ===========================================================================

#[tokio::test]
async fn article_title_with_formatting_has_facets() {
    let doc = Node::Article(Article {
        title: Some(vec![t("Hello "), em([t("world")])]),
        content: vec![p([t("Body")])],
        ..Default::default()
    });

    let codec = AtProtoCodec;
    let (json_str, _info) = codec
        .to_string(&doc, None)
        .await
        .expect("Encoding should succeed");
    let value: Value = serde_json::from_str(&json_str).expect("Output should be valid JSON");

    assert_eq!(value["title"]["text"], "Hello world");

    let facets = value["title"]["facets"]
        .as_array()
        .expect("Title with formatting should have facets");
    assert_eq!(facets.len(), 1, "Title should have exactly one facet");

    // The emphasis facet should cover bytes 6..11
    let facet = &facets[0];
    assert_eq!(facet["index"]["byteStart"], 6);
    assert_eq!(facet["index"]["byteEnd"], 11);

    // The facet should have the OXA emphasis feature type
    let features = facet["features"]
        .as_array()
        .expect("Facet should have features array");
    let feature_types: Vec<&str> = features
        .iter()
        .filter_map(|f| f["$type"].as_str())
        .collect();
    assert!(
        feature_types.contains(&OXA_EMPHASIS),
        "Should contain OXA emphasis feature type, got: {feature_types:?}"
    );
}

// ===========================================================================
// AC9: supports_to_format returns LowLoss for AtProtoJson, None otherwise
// ===========================================================================

#[test]
fn supports_to_format_atproto_json() {
    let codec = AtProtoCodec;
    let support = codec.supports_to_format(&Format::AtProtoJson);
    assert!(
        matches!(support, CodecSupport::LowLoss),
        "supports_to_format for AtProtoJson should be LowLoss, got: {support:?}"
    );
}

#[test]
fn supports_to_format_other_returns_none() {
    let codec = AtProtoCodec;
    let support = codec.supports_to_format(&Format::Json);
    assert!(
        matches!(support, CodecSupport::None),
        "supports_to_format for Json should be None, got: {support:?}"
    );
}

// ===========================================================================
// AC10: supports_from_format returns None for all formats
// ===========================================================================

#[test]
fn supports_from_format_returns_none() {
    let codec = AtProtoCodec;

    assert!(
        matches!(
            codec.supports_from_format(&Format::AtProtoJson),
            CodecSupport::None
        ),
        "supports_from_format for AtProtoJson should be None (encode-only codec)"
    );
    assert!(
        matches!(
            codec.supports_from_format(&Format::Json),
            CodecSupport::None
        ),
        "supports_from_format for Json should be None"
    );
    assert!(
        matches!(
            codec.supports_from_format(&Format::Html),
            CodecSupport::None
        ),
        "supports_from_format for Html should be None"
    );
}

// ===========================================================================
// Codec trait: name returns "atproto"
// ===========================================================================

#[test]
fn codec_name() {
    let codec = AtProtoCodec;
    assert_eq!(codec.name(), "atproto");
}
