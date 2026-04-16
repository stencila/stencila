//! Snapshot and edge-case tests for AT Protocol codec encoding.
//!
//! Phase 3 / Slices 1-2 acceptance criteria:
//!
//! AC1: Snapshot test for a representative scientific article
//! AC2: Edge-case tests (empty article, only unsupported blocks, 3+ level nested lists,
//!       triple-nested formatting, adjacent spans, zero-length facet at integration level)
//! AC3: Audit of all 18 design acceptance criteria (targeted gap-filling tests)

use insta::assert_json_snapshot;
use serde_json::Value;
use stencila_codec::{
    Codec, Losses,
    stencila_schema::{
        Article, Author, Block, DateTime, Inline, List, ListItem, ListOrder, Node, Person,
        shortcuts::{cb, em, h, li, lnk, mb, ol, p, qb, stg, sub, t, tb, ul},
    },
};
use stencila_codec_atproto::{
    AtProtoCodec,
    nsids::{BLUESKY_LINK, LEAFLET_LINK, OXA_BLOCKQUOTE, OXA_EMPHASIS, OXA_LINK, OXA_STRONG},
};

// ---------------------------------------------------------------------------
// Helper: encode a Node to JSON Value via the AtProtoCodec
// ---------------------------------------------------------------------------

async fn encode_to_value(node: &Node) -> (Value, stencila_codec::EncodeInfo) {
    let codec = AtProtoCodec;
    let (json_str, info) = codec
        .to_string(node, None)
        .await
        .expect("Encoding should succeed");
    let value: Value = serde_json::from_str(&json_str).expect("Output should be valid JSON");
    (value, info)
}

fn losses_contains(losses: &Losses, key: &str) -> bool {
    let v = serde_json::to_value(losses).expect("Losses should be serializable");
    v.as_object().is_some_and(|obj| obj.contains_key(key))
}

// ===========================================================================
// AC1: Snapshot test — representative scientific article
//
// Contains: title with emphasis, heading, mixed-formatting paragraph, code
// block with language, math block, thematic break, quote block, nested
// ordered list, unordered list, an unsupported block (Figure), Unicode
// text (emoji + CJK), and authors/abstract for loss tracking.
// ===========================================================================

#[tokio::test]
async fn snapshot_representative_scientific_article() {
    let doc = Node::Article(Article {
        title: Some(vec![t("A Study of "), em([t("Quantum")]), t(" Effects")]),
        date_published: Some(DateTime::new("2024-11-01".into())),
        authors: Some(vec![Author::Person(Person {
            family_names: Some(vec!["Smith".into()]),
            given_names: Some(vec!["Alice".into()]),
            ..Default::default()
        })]),
        r#abstract: Some(vec![p([t("This paper studies quantum effects.")])]),
        content: vec![
            // Heading
            h(2, [t("Introduction")]),
            // Mixed-formatting paragraph: plain + strong + link
            p([
                t("We study "),
                stg([t("quantum")]),
                t(" effects, see "),
                lnk([t("here")], "https://example.com"),
                t("."),
            ]),
            // Code block with language
            cb("import numpy as np", Some("python")),
            // Math block
            mb("E = mc^2", Some("tex")),
            // Thematic break
            tb(),
            // Quote block
            qb([p([t("To be or not to be.")])]),
            // Nested ordered list
            ol([
                li([t("First item")]),
                ListItem {
                    content: vec![p([t("Second item")]), ol([li([t("Nested child")])])],
                    ..Default::default()
                },
            ]),
            // Unordered list
            ul([li([t("Bullet one")]), li([t("Bullet two")])]),
            // Unicode paragraph with emoji and CJK + subscript
            p([t("🎉 Résumé: H"), sub([t("2")]), t("O 你好")]),
            // Unsupported block (Figure) — should be dropped with loss
            Block::Figure(stencila_codec::stencila_schema::Figure::new(vec![p([t(
                "Figure caption",
            )])])),
        ],
        ..Default::default()
    });

    let codec = AtProtoCodec;
    let (json_str, _info) = codec
        .to_string(&doc, None)
        .await
        .expect("Encoding should succeed");
    let value: Value = serde_json::from_str(&json_str).expect("Output should be valid JSON");

    // Redact createdAt since it's derived from a fixed date_published
    // (not dynamic), but redact for snapshot stability
    assert_json_snapshot!(value, {
        ".createdAt" => "[createdAt]"
    });
}

// ===========================================================================
// AC2a: Empty article → valid JSON with empty blocks array
// ===========================================================================

#[tokio::test]
async fn empty_article_produces_valid_json_with_empty_blocks() {
    let doc = Node::Article(Article {
        content: vec![],
        ..Default::default()
    });

    let (value, _info) = encode_to_value(&doc).await;

    let blocks = value["blocks"]
        .as_array()
        .expect("Output should have a blocks array");
    assert!(
        blocks.is_empty(),
        "Empty article should have an empty blocks array"
    );

    // Should still have createdAt
    assert!(
        value["createdAt"].is_string(),
        "Empty article should still have a createdAt field"
    );

    // Should not have title field
    assert!(
        value.get("title").is_none() || value["title"].is_null(),
        "Empty article should not have a title field"
    );
}

// ===========================================================================
// AC2b: Article with only unsupported blocks → empty blocks + losses
// ===========================================================================

#[tokio::test]
async fn article_with_only_unsupported_blocks_produces_empty_blocks_with_losses() {
    let doc = Node::Article(Article {
        content: vec![
            Block::Figure(stencila_codec::stencila_schema::Figure::new(vec![p([t(
                "fig1",
            )])])),
            Block::Figure(stencila_codec::stencila_schema::Figure::new(vec![p([t(
                "fig2",
            )])])),
        ],
        ..Default::default()
    });

    let (value, info) = encode_to_value(&doc).await;

    let blocks = value["blocks"]
        .as_array()
        .expect("Output should have a blocks array");
    assert!(
        blocks.is_empty(),
        "Article with only unsupported blocks should have empty blocks array"
    );

    assert!(
        losses_contains(&info.losses, "encode:unsupported_block_Figure"),
        "Should record unsupported block loss"
    );
}

// ===========================================================================
// AC2c: 3+ level nested lists encode correctly
// ===========================================================================

#[tokio::test]
async fn deeply_nested_list_three_levels() {
    // Level 1: ordered list
    //   Level 2: nested ordered list
    //     Level 3: nested ordered list
    let level3_item = ListItem {
        content: vec![p([t("Level 3")])],
        ..Default::default()
    };
    let level2_list = Block::List(List::new(vec![level3_item], ListOrder::Ascending));
    let level2_item = ListItem {
        content: vec![p([t("Level 2")]), level2_list],
        ..Default::default()
    };
    let level1_list = Block::List(List::new(vec![level2_item], ListOrder::Ascending));

    let doc = Node::Article(Article {
        content: vec![level1_list],
        ..Default::default()
    });

    let (value, _info) = encode_to_value(&doc).await;

    // Navigate to level 3
    let l1_children = value["blocks"][0]["children"]
        .as_array()
        .expect("Level 1 should have children");
    assert_eq!(l1_children[0]["text"], "Level 2");

    let l2_children = l1_children[0]["children"]
        .as_array()
        .expect("Level 2 item should have nested children");
    assert_eq!(l2_children[0]["text"], "Level 3");
}

// ===========================================================================
// AC2d: Triple-nested formatting — link inside emphasis inside strong
//       produces three overlapping facets with correct byte ranges
// ===========================================================================

#[tokio::test]
async fn triple_nested_formatting_link_inside_emphasis_inside_strong() {
    // strong(emphasis(link("click"))) → "click" (5 bytes)
    // Expected facets (from innermost to outermost emission order):
    //   - link facet at 0..5 (3 features: OXA+Leaflet+Bluesky link)
    //   - emphasis facet at 0..5 (2 features: OXA+Leaflet emphasis)
    //   - strong facet at 0..5 (2 features: OXA+Leaflet strong)
    let doc = Node::Article(Article {
        content: vec![p([stg([em([lnk([t("click")], "https://example.com")])])])],
        ..Default::default()
    });

    let (value, _info) = encode_to_value(&doc).await;

    let facets = value["blocks"][0]["facets"]
        .as_array()
        .expect("Should have facets for triple-nested formatting");

    assert_eq!(
        facets.len(),
        3,
        "Triple-nested formatting should produce 3 overlapping facets"
    );

    // All three facets should cover bytes 0..5
    for facet in facets {
        assert_eq!(
            facet["index"]["byteStart"]
                .as_u64()
                .expect("byteStart should be a number"),
            0,
            "All facets should start at byte 0"
        );
        assert_eq!(
            facet["index"]["byteEnd"]
                .as_u64()
                .expect("byteEnd should be a number"),
            5,
            "All facets should end at byte 5"
        );
    }

    // Collect all feature $type strings across all facets
    let empty_vec = vec![];
    let all_feature_types: Vec<&str> = facets
        .iter()
        .flat_map(|f| {
            f["features"]
                .as_array()
                .unwrap_or(&empty_vec)
                .iter()
                .filter_map(|feat| feat["$type"].as_str())
        })
        .collect();

    // Should contain link features (OXA + Leaflet + Bluesky)
    assert!(
        all_feature_types.contains(&OXA_LINK),
        "Should have OXA link feature"
    );
    assert!(
        all_feature_types.contains(&LEAFLET_LINK),
        "Should have Leaflet link feature"
    );
    assert!(
        all_feature_types.contains(&BLUESKY_LINK),
        "Should have Bluesky link feature"
    );

    // Should contain emphasis features
    assert!(
        all_feature_types.contains(&OXA_EMPHASIS),
        "Should have OXA emphasis feature"
    );

    // Should contain strong features
    assert!(
        all_feature_types.contains(&OXA_STRONG),
        "Should have OXA strong feature"
    );
}

// ===========================================================================
// AC2e: Adjacent formatting spans produce contiguous byte ranges
// ===========================================================================

#[tokio::test]
async fn adjacent_formatting_spans_contiguous_byte_ranges() {
    // em("Hello") + stg("World") → "HelloWorld" (10 bytes)
    // emphasis: 0..5, strong: 5..10
    let doc = Node::Article(Article {
        content: vec![p([em([t("Hello")]), stg([t("World")])])],
        ..Default::default()
    });

    let (value, _info) = encode_to_value(&doc).await;

    let facets = value["blocks"][0]["facets"]
        .as_array()
        .expect("Should have facets for adjacent spans");

    assert_eq!(facets.len(), 2, "Adjacent spans should produce 2 facets");

    // First facet (emphasis): 0..5
    let first_end = facets[0]["index"]["byteEnd"]
        .as_u64()
        .expect("byteEnd should be a number");
    assert_eq!(
        facets[0]["index"]["byteStart"].as_u64().expect("byteStart"),
        0
    );
    assert_eq!(first_end, 5);

    // Second facet (strong): 5..10 — should start exactly where the first ended
    let second_start = facets[1]["index"]["byteStart"].as_u64().expect("byteStart");
    assert_eq!(
        second_start, first_end,
        "Adjacent spans should have contiguous byte ranges (no gap)"
    );
    assert_eq!(facets[1]["index"]["byteEnd"].as_u64().expect("byteEnd"), 10);
}

// ===========================================================================
// AC2f: Zero-length facet at integration level (article encoding) is omitted
// ===========================================================================

#[tokio::test]
async fn zero_length_facet_at_integration_level_is_omitted() {
    // An article containing a paragraph with empty emphasis — should not
    // produce any facets in the encoded output
    let doc = Node::Article(Article {
        content: vec![p([t("before"), em(Vec::<Inline>::new()), t("after")])],
        ..Default::default()
    });

    let (value, _info) = encode_to_value(&doc).await;

    let block = &value["blocks"][0];
    assert_eq!(block["text"], "beforeafter");

    // Should have no facets (empty emphasis produces zero-length span → omitted)
    let facets = block.get("facets");
    assert!(
        facets.is_none()
            || facets
                .and_then(|v| v.as_array())
                .is_some_and(|a| a.is_empty()),
        "Zero-length emphasis should not produce any facets in encoded article output"
    );
}

// ===========================================================================
// AC3: Design acceptance criteria audit — targeted gap-filling tests
//
// Most design ACs are already covered by existing tests in facets.rs,
// blocks.rs, and encode.rs. The tests below fill identified gaps.
// ===========================================================================

// ---------------------------------------------------------------------------
// AC8 gap: Format path resolution — verify .atproto.json file extension
// resolves to Format::AtProtoJson (tested at format crate level)
// ---------------------------------------------------------------------------

#[test]
fn format_path_resolution_atproto_json() {
    use std::path::PathBuf;
    use stencila_codec::stencila_format::Format;

    let format = Format::from_path(&PathBuf::from("document.atproto.json"));
    assert_eq!(
        format,
        Format::AtProtoJson,
        ".atproto.json extension should resolve to Format::AtProtoJson"
    );
}

#[test]
fn format_name_resolution_atproto_json() {
    use stencila_codec::stencila_format::Format;

    let format = Format::from_name("atproto.json");
    assert_eq!(
        format,
        Format::AtProtoJson,
        "Name 'atproto.json' should resolve to Format::AtProtoJson"
    );

    let format2 = Format::from_name("atprotojson");
    assert_eq!(
        format2,
        Format::AtProtoJson,
        "Name 'atprotojson' should resolve to Format::AtProtoJson"
    );
}

// ---------------------------------------------------------------------------
// AC17 gap: Verify TODO comments exist on provisional NSIDs
// (Non-test deliverable — documented here for completeness but not tested
//  via automated tests. The nsids.rs file should contain TODO comments.)
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// AC14 supplement: QuoteBlock single-paragraph preserves text correctly
// (blocks.rs tests cover multi-paragraph; this confirms single-paragraph
//  at the full-article integration level)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn quote_block_single_paragraph_at_article_level() {
    let doc = Node::Article(Article {
        content: vec![qb([p([t("A famous quote.")])])],
        ..Default::default()
    });

    let (value, _info) = encode_to_value(&doc).await;

    let block = &value["blocks"][0];
    assert_eq!(
        block["$type"], OXA_BLOCKQUOTE,
        "QuoteBlock should have blockquote $type"
    );
    assert_eq!(block["text"], "A famous quote.");
}

// ---------------------------------------------------------------------------
// AC15 supplement: Mixed ordered/unordered nested list at article level
// records loss
// ---------------------------------------------------------------------------

#[tokio::test]
async fn mixed_nesting_list_at_article_level_records_loss() {
    // Ordered parent with unordered nested child
    let nested_item = ListItem {
        content: vec![p([t("Parent")]), ul([li([t("Unordered child")])])],
        ..Default::default()
    };
    let doc = Node::Article(Article {
        content: vec![Block::List(List::new(
            vec![nested_item],
            ListOrder::Ascending,
        ))],
        ..Default::default()
    });

    let (_value, info) = encode_to_value(&doc).await;

    assert!(
        losses_contains(&info.losses, "encode:list_mixed_nesting_type"),
        "Mixed nesting type should record loss at article level"
    );
}
