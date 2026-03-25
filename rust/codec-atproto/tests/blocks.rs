//! Tests for the block-level encoder in `blocks.rs`.
//!
//! Each test covers one or more acceptance criteria from Phase 2 / Slice 1.

use pretty_assertions::assert_eq;
use serde_json::Value;
use stencila_codec::{
    Losses,
    stencila_schema::{
        Block, Heading, List, ListItem, ListOrder, Paragraph,
        shortcuts::{cb, em, h, li, mb, ol, p, qb, stg, t, tb, ul},
    },
};
use stencila_codec_atproto::{
    blocks::encode_block,
    nsids::{
        OXA_CODE, OXA_EMPHASIS, OXA_HEADING, OXA_MATH, OXA_ORDERED_LIST, OXA_PARAGRAPH, OXA_STRONG,
        OXA_THEMATIC_BREAK, OXA_UNORDERED_LIST,
    },
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
// AC1: Paragraph with formatted inlines encodes to correct $type, text,
//      and facets structure
// ===========================================================================

#[test]
fn paragraph_plain_text_encodes_to_paragraph_block() {
    let block = p([t("Hello world")]);
    let mut losses = Losses::none();
    let result = encode_block(&block, &mut losses);

    let value = result.expect("Paragraph should encode to Some");
    assert_eq!(value["$type"], OXA_PARAGRAPH);
    assert_eq!(value["text"], "Hello world");
    // No facets for plain text
    let facets = value.get("facets");
    assert!(
        facets.is_none()
            || facets
                .and_then(|v| v.as_array())
                .is_some_and(|a| a.is_empty()),
        "Plain text paragraph should have no facets"
    );
    assert!(losses.is_empty(), "No losses for plain paragraph");
}

#[test]
fn paragraph_with_emphasis_encodes_text_and_facets() {
    // "Hello " (6 bytes) + emphasis("world") (5 bytes) → facet at bytes 6..11
    let block = p([t("Hello "), em([t("world")])]);
    let mut losses = Losses::none();
    let result = encode_block(&block, &mut losses);

    let value = result.expect("Paragraph should encode to Some");
    assert_eq!(value["$type"], OXA_PARAGRAPH);
    assert_eq!(value["text"], "Hello world");

    let facets = value["facets"]
        .as_array()
        .expect("Should have facets array");
    assert_eq!(facets.len(), 1, "Single emphasis should produce one facet");

    // Verify byte offsets for the emphasis facet
    let facet = &facets[0];
    assert_eq!(facet["index"]["byteStart"], 6);
    assert_eq!(facet["index"]["byteEnd"], 11);

    // Verify the facet has the expected OXA emphasis feature type
    let features = facet["features"]
        .as_array()
        .expect("Facet should have features array");
    let feature_types: Vec<&str> = features
        .iter()
        .filter_map(|f| f["$type"].as_str())
        .collect();
    assert!(
        feature_types.contains(&OXA_EMPHASIS),
        "Should contain OXA emphasis feature, got: {feature_types:?}"
    );
    assert!(losses.is_empty());
}

// ===========================================================================
// AC2: Heading encodes with level and text/facets
// ===========================================================================

#[test]
fn heading_encodes_with_level_and_text() {
    let block = h(2, [t("My Heading")]);
    let mut losses = Losses::none();
    let result = encode_block(&block, &mut losses);

    let value = result.expect("Heading should encode to Some");
    assert_eq!(value["$type"], OXA_HEADING);
    assert_eq!(value["level"], 2);
    assert_eq!(value["text"], "My Heading");
    assert!(losses.is_empty());
}

#[test]
fn heading_with_formatted_inlines_has_facets() {
    // "Say " (4 bytes) + strong("bold") (4 bytes) → facet at bytes 4..8
    let block = h(3, [t("Say "), stg([t("bold")])]);
    let mut losses = Losses::none();
    let result = encode_block(&block, &mut losses);

    let value = result.expect("Heading should encode to Some");
    assert_eq!(value["$type"], OXA_HEADING);
    assert_eq!(value["level"], 3);
    assert_eq!(value["text"], "Say bold");

    let facets = value["facets"]
        .as_array()
        .expect("Should have facets array for formatted heading");
    assert_eq!(facets.len(), 1, "Single strong should produce one facet");

    // Verify byte offsets for the strong facet
    let facet = &facets[0];
    assert_eq!(facet["index"]["byteStart"], 4);
    assert_eq!(facet["index"]["byteEnd"], 8);

    // Verify the facet has the expected OXA strong feature type
    let features = facet["features"]
        .as_array()
        .expect("Facet should have features array");
    let feature_types: Vec<&str> = features
        .iter()
        .filter_map(|f| f["$type"].as_str())
        .collect();
    assert!(
        feature_types.contains(&OXA_STRONG),
        "Should contain OXA strong feature, got: {feature_types:?}"
    );
    assert!(losses.is_empty());
}

// ===========================================================================
// AC3: CodeBlock encodes with value and optional language
// ===========================================================================

#[test]
fn code_block_encodes_with_value_and_language() {
    let block = cb("print('hello')", Some("python"));
    let mut losses = Losses::none();
    let result = encode_block(&block, &mut losses);

    let value = result.expect("CodeBlock should encode to Some");
    assert_eq!(value["$type"], OXA_CODE);
    assert_eq!(value["value"], "print('hello')");
    assert_eq!(value["language"], "python");
    assert!(losses.is_empty());
}

#[test]
fn code_block_encodes_without_language_when_none() {
    let block = cb("x = 1", None::<String>);
    let mut losses = Losses::none();
    let result = encode_block(&block, &mut losses);

    let value = result.expect("CodeBlock should encode to Some");
    assert_eq!(value["$type"], OXA_CODE);
    assert_eq!(value["value"], "x = 1");
    // language should be absent or null
    assert!(
        value.get("language").is_none() || value["language"].is_null(),
        "CodeBlock without language should not have a language field"
    );
    assert!(losses.is_empty());
}

// ===========================================================================
// AC4: ThematicBreak encodes with only $type
// ===========================================================================

#[test]
fn thematic_break_encodes_with_type_only() {
    let block = tb();
    let mut losses = Losses::none();
    let result = encode_block(&block, &mut losses);

    let value = result.expect("ThematicBreak should encode to Some");
    assert_eq!(value["$type"], OXA_THEMATIC_BREAK);

    // Should only have $type key (no other fields)
    let obj = value.as_object().expect("Should be a JSON object");
    assert_eq!(
        obj.len(),
        1,
        "ThematicBreak should have only $type, got keys: {:?}",
        obj.keys().collect::<Vec<_>>()
    );
    assert!(losses.is_empty());
}

// ===========================================================================
// AC5: MathBlock encodes with tex field
// ===========================================================================

#[test]
fn math_block_encodes_with_type_and_tex_field() {
    let block = mb("E = mc^2", Some("tex"));
    let mut losses = Losses::none();
    let result = encode_block(&block, &mut losses);

    let value = result.expect("MathBlock should encode to Some");
    assert_eq!(value["$type"], OXA_MATH);
    assert_eq!(value["tex"], "E = mc^2");

    // Should only contain $type and tex (no extra fields)
    let obj = value.as_object().expect("Should be a JSON object");
    assert!(
        obj.len() <= 3,
        "MathBlock should not have unexpected extra fields, got keys: {:?}",
        obj.keys().collect::<Vec<_>>()
    );
}

// ===========================================================================
// AC6: Unsupported block type returns None and records loss
// ===========================================================================

#[test]
fn unsupported_block_returns_none_and_records_loss() {
    // Figure is not in the supported block types for AT Protocol encoding
    let block = Block::Figure(stencila_codec::stencila_schema::Figure::new(vec![p([t(
        "caption",
    )])]));
    let mut losses = Losses::none();
    let result = encode_block(&block, &mut losses);

    assert!(
        result.is_none(),
        "Unsupported block type should return None"
    );
    assert!(!losses.is_empty(), "Unsupported block should record a loss");
    assert!(
        losses_contains(&losses, "encode:unsupported_block_Figure"),
        "Loss key should be encode:unsupported_block_Figure, got: {:?}",
        losses_to_value(&losses)
    );
}

// ===========================================================================
// AC7: QuoteBlock with two paragraphs encodes to \n-joined text with correct
//      adjusted facet offsets and records partial loss
// ===========================================================================

#[test]
fn quote_block_two_paragraphs_joined_with_newline_and_records_partial_loss() {
    let block = qb([p([t("First paragraph")]), p([t("Second paragraph")])]);
    let mut losses = Losses::none();
    let result = encode_block(&block, &mut losses);

    let value = result.expect("QuoteBlock should encode to Some");
    // Text should be the two paragraphs joined with \n
    assert_eq!(value["text"], "First paragraph\nSecond paragraph");

    // QuoteBlock flattening to richtext is a lossy operation (block structure lost)
    // so a partial loss should be recorded
    assert!(
        !losses.is_empty(),
        "QuoteBlock paragraph flattening should record a partial loss, got: {:?}",
        losses_to_value(&losses)
    );
}

#[test]
fn quote_block_with_formatted_text_has_adjusted_facet_offsets() {
    // First paragraph: "Hello " + emphasis("world") = "Hello world" (11 bytes)
    // Second paragraph: "Say " + strong("bold") = "Say bold" (8 bytes)
    // Joined: "Hello world\nSay bold" (20 bytes, \n at byte 11)
    let block = qb([
        p([t("Hello "), em([t("world")])]),
        p([t("Say "), stg([t("bold")])]),
    ]);
    let mut losses = Losses::none();
    let result = encode_block(&block, &mut losses);

    let value = result.expect("QuoteBlock should encode to Some");
    assert_eq!(value["text"], "Hello world\nSay bold");

    let facets = value["facets"]
        .as_array()
        .expect("QuoteBlock with formatted text should have facets");
    assert_eq!(
        facets.len(),
        2,
        "Should have two facets (emphasis + strong)"
    );

    // First facet (emphasis): byte range 6..11 (same as in first paragraph)
    let first_facet = &facets[0];
    assert_eq!(
        first_facet["index"]["byteStart"]
            .as_u64()
            .expect("byteStart should be a number"),
        6
    );
    assert_eq!(
        first_facet["index"]["byteEnd"]
            .as_u64()
            .expect("byteEnd should be a number"),
        11
    );

    // Second facet (strong): byte range should be offset by length of first paragraph + \n
    // "Hello world\n" = 12 bytes, "Say " = 4 bytes → strong starts at 16, ends at 20
    let second_facet = &facets[1];
    assert_eq!(
        second_facet["index"]["byteStart"]
            .as_u64()
            .expect("byteStart should be a number"),
        16
    );
    assert_eq!(
        second_facet["index"]["byteEnd"]
            .as_u64()
            .expect("byteEnd should be a number"),
        20
    );
}

// ===========================================================================
// AC8: QuoteBlock with non-paragraph child records loss
// ===========================================================================

#[test]
fn quote_block_with_non_paragraph_child_records_loss() {
    // QuoteBlock containing a heading (non-paragraph) should record a loss
    let block = qb([p([t("Normal paragraph")]), h(2, [t("A heading")])]);
    let mut losses = Losses::none();
    let _result = encode_block(&block, &mut losses);

    assert!(
        losses_contains(&losses, "encode:blockquote_non_paragraph_child"),
        "QuoteBlock with non-paragraph child should record encode:blockquote_non_paragraph_child loss, got: {:?}",
        losses_to_value(&losses)
    );
}

// ===========================================================================
// AC9: Ordered List with nested items encodes with correct $type, children
//      structure, and startIndex
// ===========================================================================

#[test]
fn ordered_list_encodes_with_type_children_and_start_index() {
    let block = ol([li([t("First item")]), li([t("Second item")])]);
    let mut losses = Losses::none();
    let result = encode_block(&block, &mut losses);

    let value = result.expect("Ordered list should encode to Some");

    // Verify $type is the ordered list block type
    assert_eq!(
        value["$type"], OXA_ORDERED_LIST,
        "Ordered list should have the orderedList $type"
    );

    // Ordered list should have a startIndex (typically 1)
    assert_eq!(
        value["startIndex"], 1,
        "Ordered list startIndex should be 1"
    );

    // Verify children array with expected structure
    let children = value["children"]
        .as_array()
        .expect("List should have children array");
    assert_eq!(children.len(), 2);

    // Each child should have text content from the list items
    assert_eq!(children[0]["text"], "First item");
    assert_eq!(children[1]["text"], "Second item");
}

#[test]
fn ordered_list_with_nested_sub_list_encodes_children() {
    // An ordered list item containing a nested ordered sub-list
    let nested_item = ListItem {
        content: vec![p([t("Parent item")]), ol([li([t("Nested child")])])],
        ..Default::default()
    };
    let block = Block::List(List::new(vec![nested_item], ListOrder::Ascending));
    let mut losses = Losses::none();
    let result = encode_block(&block, &mut losses);

    let value = result.expect("Ordered list with nesting should encode to Some");

    // Parent list should have the ordered list $type
    assert_eq!(
        value["$type"], OXA_ORDERED_LIST,
        "Parent list should have orderedList $type"
    );

    let children = value["children"]
        .as_array()
        .expect("List should have children array");
    assert_eq!(children.len(), 1);

    // The parent item should have text for its paragraph content
    assert_eq!(children[0]["text"], "Parent item");

    // The parent item should carry nested children from the sub-list
    let nested_children = children[0]["children"]
        .as_array()
        .expect("Nested list item should have children array");
    assert_eq!(nested_children.len(), 1);
    assert_eq!(nested_children[0]["text"], "Nested child");
}

// ===========================================================================
// AC10: Unordered List encodes correctly
// ===========================================================================

#[test]
fn unordered_list_encodes_with_type_and_children() {
    let block = ul([li([t("Bullet one")]), li([t("Bullet two")])]);
    let mut losses = Losses::none();
    let result = encode_block(&block, &mut losses);

    let value = result.expect("Unordered list should encode to Some");

    // Verify $type is the unordered list block type
    assert_eq!(
        value["$type"], OXA_UNORDERED_LIST,
        "Unordered list should have the unorderedList $type"
    );

    // Unordered list should NOT have a startIndex
    assert!(
        value.get("startIndex").is_none() || value["startIndex"].is_null(),
        "Unordered list should not have a startIndex"
    );

    // Verify children array with expected content
    let children = value["children"]
        .as_array()
        .expect("List should have children array");
    assert_eq!(children.len(), 2);

    // Each child should have text content
    assert_eq!(children[0]["text"], "Bullet one");
    assert_eq!(children[1]["text"], "Bullet two");
}

// ===========================================================================
// AC11: Multi-block list item records loss
// ===========================================================================

#[test]
fn multi_block_list_item_records_loss() {
    // A list item with two blocks (paragraph + code block) should record a loss
    let item = ListItem {
        content: vec![p([t("Paragraph text")]), cb("some code", None::<String>)],
        ..Default::default()
    };
    let block = Block::List(List::new(vec![item], ListOrder::Unordered));
    let mut losses = Losses::none();
    let _result = encode_block(&block, &mut losses);

    assert!(
        losses_contains(&losses, "encode:list_item_extra_blocks"),
        "Multi-block list item should record encode:list_item_extra_blocks loss, got: {:?}",
        losses_to_value(&losses)
    );
}

// ===========================================================================
// AC12: Mixed nesting type records loss
// ===========================================================================

#[test]
fn mixed_nesting_type_records_loss() {
    // A list item that contains a nested list of a different ordering type
    // e.g., an unordered list item containing an ordered sub-list
    let nested_item = ListItem {
        content: vec![p([t("Parent item")]), ol([li([t("Nested ordered")])])],
        ..Default::default()
    };
    let block = Block::List(List::new(vec![nested_item], ListOrder::Unordered));
    let mut losses = Losses::none();
    let _result = encode_block(&block, &mut losses);

    assert!(
        losses_contains(&losses, "encode:list_mixed_nesting_type"),
        "Mixed nesting type should record encode:list_mixed_nesting_type loss, got: {:?}",
        losses_to_value(&losses)
    );
}

// ===========================================================================
// AC13: Block with id/classes records encode:dropped_property_id and
//       encode:dropped_property_classes losses
// ===========================================================================

#[test]
fn paragraph_with_id_records_dropped_property_id_loss() {
    let block = Block::Paragraph(Paragraph {
        id: Some("my-para-id".to_string()),
        content: vec![t("Content with id")],
        ..Default::default()
    });
    let mut losses = Losses::none();
    let _result = encode_block(&block, &mut losses);

    assert!(
        losses_contains(&losses, "encode:dropped_property_id"),
        "Paragraph with id should record encode:dropped_property_id loss, got: {:?}",
        losses_to_value(&losses)
    );
}

#[test]
fn heading_with_id_records_dropped_property_id_loss() {
    let block = Block::Heading(Heading {
        id: Some("heading-id".to_string()),
        level: 1,
        content: vec![t("Title")],
        ..Default::default()
    });
    let mut losses = Losses::none();
    let _result = encode_block(&block, &mut losses);

    assert!(
        losses_contains(&losses, "encode:dropped_property_id"),
        "Heading with id should record encode:dropped_property_id loss, got: {:?}",
        losses_to_value(&losses)
    );
}

#[test]
fn paragraph_without_id_does_not_record_dropped_id_loss() {
    let block = p([t("No id here")]);
    let mut losses = Losses::none();
    let _result = encode_block(&block, &mut losses);

    assert!(
        !losses_contains(&losses, "encode:dropped_property_id"),
        "Paragraph without id should NOT record encode:dropped_property_id loss, got: {:?}",
        losses_to_value(&losses)
    );
}

#[test]
fn block_with_classes_records_dropped_property_classes_loss() {
    // StyledBlock has a class_list field — use it to verify classes loss tracking
    use stencila_codec::stencila_schema::{Cord, StyledBlock, StyledBlockOptions};
    let block = Block::StyledBlock(StyledBlock {
        code: Cord::from("color: red"),
        content: vec![p([t("Styled content")])],
        options: Box::new(StyledBlockOptions {
            class_list: Some("highlight important".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    });
    let mut losses = Losses::none();
    let _result = encode_block(&block, &mut losses);

    assert!(
        losses_contains(&losses, "encode:dropped_property_classes"),
        "Block with classes should record encode:dropped_property_classes loss, got: {:?}",
        losses_to_value(&losses)
    );
}
