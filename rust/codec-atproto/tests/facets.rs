//! Tests for the inline-tree-to-facet flattening algorithm in `facets.rs`.
//!
//! Each test covers one or more acceptance criteria from Phase 1 / Slice 2.

use pretty_assertions::assert_eq;
use stencila_codec::{
    Losses,
    stencila_schema::{
        Inline,
        shortcuts::{ci, em, lnk, qi, stg, stk, sub, sup, t, u},
    },
};
use stencila_codec_atproto::{
    facets::flatten_inlines,
    nsids::{
        BLUESKY_LINK, LEAFLET_BOLD, LEAFLET_CODE, LEAFLET_ITALIC, LEAFLET_LINK,
        LEAFLET_STRIKETHROUGH, LEAFLET_UNDERLINE, OXA_EMPHASIS, OXA_INLINE_CODE, OXA_LINK,
        OXA_STRIKETHROUGH, OXA_STRONG, OXA_SUBSCRIPT, OXA_SUPERSCRIPT, OXA_UNDERLINE,
    },
};

// ---------------------------------------------------------------------------
// AC1: Plain text only — no facets emitted
// ---------------------------------------------------------------------------

#[test]
fn plain_text_no_facets() {
    let inlines = vec![t("Hello world")];
    let mut losses = Losses::none();
    let rt = flatten_inlines(&inlines, &mut losses);

    assert_eq!(rt.text, "Hello world");
    assert!(rt.facets.is_empty(), "Plain text should produce no facets");
    assert!(losses.is_empty(), "No losses for plain text");
}

#[test]
fn multiple_plain_text_segments() {
    let inlines = vec![t("Hello "), t("world")];
    let mut losses = Losses::none();
    let rt = flatten_inlines(&inlines, &mut losses);

    assert_eq!(rt.text, "Hello world");
    assert!(rt.facets.is_empty());
}

// ---------------------------------------------------------------------------
// AC2: Single emphasis and strong with correct byte offsets and multi-family
//      features (OXA + Leaflet)
// ---------------------------------------------------------------------------

#[test]
fn single_emphasis_byte_offsets_and_features() {
    // "Hello *world*" → text = "Hello world", one facet at bytes 6..11
    let inlines = vec![t("Hello "), em([t("world")])];
    let mut losses = Losses::none();
    let rt = flatten_inlines(&inlines, &mut losses);

    assert_eq!(rt.text, "Hello world");
    assert_eq!(rt.facets.len(), 1);

    let facet = &rt.facets[0];
    assert_eq!(facet.index.byte_start, 6);
    assert_eq!(facet.index.byte_end, 11);

    // Must emit OXA emphasis + Leaflet italic (two features)
    assert_eq!(facet.features.len(), 2);
    let type_strs: Vec<&str> = facet.features.iter().map(|f| f.type_str).collect();
    assert!(
        type_strs.contains(&OXA_EMPHASIS),
        "Should contain OXA emphasis feature, got: {type_strs:?}"
    );
    assert!(
        type_strs.contains(&LEAFLET_ITALIC),
        "Should contain Leaflet italic feature, got: {type_strs:?}"
    );
    assert!(losses.is_empty());
}

#[test]
fn single_strong_byte_offsets_and_features() {
    // "Say **bold** end" → text = "Say bold end", facet at bytes 4..8
    let inlines = vec![t("Say "), stg([t("bold")]), t(" end")];
    let mut losses = Losses::none();
    let rt = flatten_inlines(&inlines, &mut losses);

    assert_eq!(rt.text, "Say bold end");
    assert_eq!(rt.facets.len(), 1);

    let facet = &rt.facets[0];
    assert_eq!(facet.index.byte_start, 4);
    assert_eq!(facet.index.byte_end, 8);

    let type_strs: Vec<&str> = facet.features.iter().map(|f| f.type_str).collect();
    assert!(type_strs.contains(&OXA_STRONG));
    assert!(type_strs.contains(&LEAFLET_BOLD));
    assert!(losses.is_empty());
}

// ---------------------------------------------------------------------------
// AC3: Nested emphasis inside strong — overlapping facets with correct ranges
// ---------------------------------------------------------------------------

#[test]
fn nested_emphasis_inside_strong() {
    // "**bold and *italic***" → text = "bold and italic"
    // strong facet: bytes 0..15, emphasis facet: bytes 9..15
    let inlines = vec![stg([t("bold and "), em([t("italic")])])];
    let mut losses = Losses::none();
    let rt = flatten_inlines(&inlines, &mut losses);

    assert_eq!(rt.text, "bold and italic");
    assert_eq!(rt.facets.len(), 2, "Should have strong + emphasis facets");

    // Find the strong facet (covers entire range)
    let strong_facet = rt
        .facets
        .iter()
        .find(|f| f.features.iter().any(|feat| feat.type_str == OXA_STRONG))
        .expect("Should have a strong facet");
    assert_eq!(strong_facet.index.byte_start, 0);
    assert_eq!(strong_facet.index.byte_end, 15);

    // Find the emphasis facet (covers nested part)
    let em_facet = rt
        .facets
        .iter()
        .find(|f| f.features.iter().any(|feat| feat.type_str == OXA_EMPHASIS))
        .expect("Should have an emphasis facet");
    assert_eq!(em_facet.index.byte_start, 9);
    assert_eq!(em_facet.index.byte_end, 15);
    assert!(losses.is_empty());
}

// ---------------------------------------------------------------------------
// AC4: CodeInline — text extraction and code facet (OXA + Leaflet)
// ---------------------------------------------------------------------------

#[test]
fn code_inline_text_and_facet() {
    // "Use `foo()` here" → text = "Use foo() here", facet at bytes 4..9
    let inlines = vec![t("Use "), ci("foo()"), t(" here")];
    let mut losses = Losses::none();
    let rt = flatten_inlines(&inlines, &mut losses);

    assert_eq!(rt.text, "Use foo() here");
    assert_eq!(rt.facets.len(), 1);

    let facet = &rt.facets[0];
    assert_eq!(facet.index.byte_start, 4);
    assert_eq!(facet.index.byte_end, 9);

    let type_strs: Vec<&str> = facet.features.iter().map(|f| f.type_str).collect();
    assert!(type_strs.contains(&OXA_INLINE_CODE));
    assert!(type_strs.contains(&LEAFLET_CODE));
    assert!(losses.is_empty());
}

// ---------------------------------------------------------------------------
// AC5: Subscript/Superscript — OXA-only features (no Leaflet analogue)
// ---------------------------------------------------------------------------

#[test]
fn subscript_oxa_only_feature() {
    let inlines = vec![t("H"), sub([t("2")]), t("O")];
    let mut losses = Losses::none();
    let rt = flatten_inlines(&inlines, &mut losses);

    assert_eq!(rt.text, "H2O");
    assert_eq!(rt.facets.len(), 1);

    let facet = &rt.facets[0];
    assert_eq!(facet.index.byte_start, 1);
    assert_eq!(facet.index.byte_end, 2);
    assert_eq!(facet.features.len(), 1, "Subscript should be OXA-only");
    assert_eq!(facet.features[0].type_str, OXA_SUBSCRIPT);
    assert!(losses.is_empty());
}

#[test]
fn superscript_oxa_only_feature() {
    let inlines = vec![t("x"), sup([t("2")])];
    let mut losses = Losses::none();
    let rt = flatten_inlines(&inlines, &mut losses);

    assert_eq!(rt.text, "x2");
    assert_eq!(rt.facets.len(), 1);

    let facet = &rt.facets[0];
    assert_eq!(facet.index.byte_start, 1);
    assert_eq!(facet.index.byte_end, 2);
    assert_eq!(facet.features.len(), 1, "Superscript should be OXA-only");
    assert_eq!(facet.features[0].type_str, OXA_SUPERSCRIPT);
    assert!(losses.is_empty());
}

// ---------------------------------------------------------------------------
// AC6: Strikethrough/Underline — OXA + Leaflet features
// ---------------------------------------------------------------------------

#[test]
fn strikethrough_oxa_and_leaflet() {
    let inlines = vec![t("not "), stk([t("deleted")])];
    let mut losses = Losses::none();
    let rt = flatten_inlines(&inlines, &mut losses);

    assert_eq!(rt.text, "not deleted");
    assert_eq!(rt.facets.len(), 1);

    let facet = &rt.facets[0];
    assert_eq!(facet.index.byte_start, 4);
    assert_eq!(facet.index.byte_end, 11);
    assert_eq!(facet.features.len(), 2);
    let type_strs: Vec<&str> = facet.features.iter().map(|f| f.type_str).collect();
    assert!(type_strs.contains(&OXA_STRIKETHROUGH));
    assert!(type_strs.contains(&LEAFLET_STRIKETHROUGH));
    assert!(losses.is_empty());
}

#[test]
fn underline_oxa_and_leaflet() {
    let inlines = vec![u([t("underlined")])];
    let mut losses = Losses::none();
    let rt = flatten_inlines(&inlines, &mut losses);

    assert_eq!(rt.text, "underlined");
    assert_eq!(rt.facets.len(), 1);

    let facet = &rt.facets[0];
    assert_eq!(facet.index.byte_start, 0);
    assert_eq!(facet.index.byte_end, 10);
    assert_eq!(facet.features.len(), 2);
    let type_strs: Vec<&str> = facet.features.iter().map(|f| f.type_str).collect();
    assert!(type_strs.contains(&OXA_UNDERLINE));
    assert!(type_strs.contains(&LEAFLET_UNDERLINE));
    assert!(losses.is_empty());
}

// ---------------------------------------------------------------------------
// AC7: Link with HTTPS URL — three-family features (OXA + Leaflet + Bluesky)
// ---------------------------------------------------------------------------

#[test]
fn link_https_three_family_features() {
    let inlines = vec![
        t("See "),
        lnk([t("example")], "https://example.com"),
        t(" here"),
    ];
    let mut losses = Losses::none();
    let rt = flatten_inlines(&inlines, &mut losses);

    assert_eq!(rt.text, "See example here");
    assert_eq!(rt.facets.len(), 1);

    let facet = &rt.facets[0];
    assert_eq!(facet.index.byte_start, 4);
    assert_eq!(facet.index.byte_end, 11);
    assert_eq!(
        facet.features.len(),
        3,
        "HTTPS link should emit three-family features"
    );

    let type_strs: Vec<&str> = facet.features.iter().map(|f| f.type_str).collect();
    assert!(type_strs.contains(&OXA_LINK));
    assert!(type_strs.contains(&LEAFLET_LINK));
    assert!(type_strs.contains(&BLUESKY_LINK));

    // Verify URI is stored in the features' extra data
    for feature in &facet.features {
        if let Some(extra) = &feature.extra {
            let uri = extra.get("uri").and_then(|v| v.as_str());
            assert_eq!(
                uri,
                Some("https://example.com"),
                "Link feature should carry the URI"
            );
        }
    }
    assert!(losses.is_empty());
}

// ---------------------------------------------------------------------------
// AC8: Link with relative path — two-family features (OXA + Leaflet, no Bluesky)
// ---------------------------------------------------------------------------

#[test]
fn link_relative_path_two_family_features() {
    let inlines = vec![lnk([t("local doc")], "./other.md")];
    let mut losses = Losses::none();
    let rt = flatten_inlines(&inlines, &mut losses);

    assert_eq!(rt.text, "local doc");
    assert_eq!(rt.facets.len(), 1);

    let facet = &rt.facets[0];
    assert_eq!(facet.index.byte_start, 0);
    assert_eq!(facet.index.byte_end, 9);
    assert_eq!(
        facet.features.len(),
        2,
        "Relative link should emit two-family features (no Bluesky)"
    );

    let type_strs: Vec<&str> = facet.features.iter().map(|f| f.type_str).collect();
    assert!(type_strs.contains(&OXA_LINK));
    assert!(type_strs.contains(&LEAFLET_LINK));
    assert!(
        !type_strs.contains(&BLUESKY_LINK),
        "Bluesky link should NOT be emitted for relative paths"
    );
    assert!(losses.is_empty());
}

#[test]
fn link_http_three_family_features() {
    // http:// (not just https://) should also emit Bluesky link
    let inlines = vec![lnk([t("click")], "http://example.com")];
    let mut losses = Losses::none();
    let rt = flatten_inlines(&inlines, &mut losses);

    assert_eq!(rt.text, "click");
    let facet = &rt.facets[0];
    let type_strs: Vec<&str> = facet.features.iter().map(|f| f.type_str).collect();
    assert!(
        type_strs.contains(&BLUESKY_LINK),
        "http:// links should also emit Bluesky feature"
    );
}

// ---------------------------------------------------------------------------
// AC9: Multi-byte Unicode characters with formatting — correct byte offsets
// ---------------------------------------------------------------------------

#[test]
fn unicode_emoji_with_emphasis_byte_offsets() {
    // "🎉 " is 4+1 = 5 bytes, then "party" is 5 bytes
    let inlines = vec![t("🎉 "), em([t("party")])];
    let mut losses = Losses::none();
    let rt = flatten_inlines(&inlines, &mut losses);

    assert_eq!(rt.text, "🎉 party");
    assert_eq!(rt.facets.len(), 1);

    let facet = &rt.facets[0];
    // "🎉" is 4 bytes (U+1F389), " " is 1 byte → emphasis starts at byte 5
    assert_eq!(facet.index.byte_start, 5);
    assert_eq!(facet.index.byte_end, 10);
}

#[test]
fn unicode_cjk_with_strong_byte_offsets() {
    // "你好" = 6 bytes (3 bytes per CJK character), then " " = 1 byte, then "world" = 5 bytes
    let inlines = vec![t("你好 "), stg([t("world")])];
    let mut losses = Losses::none();
    let rt = flatten_inlines(&inlines, &mut losses);

    assert_eq!(rt.text, "你好 world");
    assert_eq!(rt.facets.len(), 1);

    let facet = &rt.facets[0];
    // "你好" = 6 bytes, " " = 1 byte → strong starts at byte 7
    assert_eq!(facet.index.byte_start, 7);
    assert_eq!(facet.index.byte_end, 12);
}

#[test]
fn unicode_mixed_formatting_byte_offsets() {
    // "café" has "caf" = 3 bytes, "é" = 2 bytes → "café" = 5 bytes
    // Then " " = 1 byte, then emphasis on "über" → "ü" = 2 bytes + "ber" = 3 bytes = 5 bytes
    let inlines = vec![t("café "), em([t("über")])];
    let mut losses = Losses::none();
    let rt = flatten_inlines(&inlines, &mut losses);

    assert_eq!(rt.text, "café über");
    assert_eq!(rt.facets.len(), 1);

    let facet = &rt.facets[0];
    assert_eq!(facet.index.byte_start, 6); // "café " = 5+1 = 6 bytes
    assert_eq!(facet.index.byte_end, 11); // "über" = 5 bytes → 6+5 = 11
}

// ---------------------------------------------------------------------------
// AC10: Zero-length facets are omitted (empty emphasis, empty link text)
// ---------------------------------------------------------------------------

#[test]
fn zero_length_emphasis_omitted() {
    // Empty emphasis span should not produce a facet
    let inlines: Vec<Inline> = vec![t("before"), em(Vec::new()), t("after")];
    let mut losses = Losses::none();
    let rt = flatten_inlines(&inlines, &mut losses);

    assert_eq!(rt.text, "beforeafter");
    assert!(
        rt.facets.is_empty(),
        "Empty emphasis should not produce a facet"
    );
}

#[test]
fn zero_length_link_text_omitted() {
    // Link with no visible text should not produce a facet
    let inlines: Vec<Inline> = vec![t("start"), lnk(Vec::new(), "https://example.com"), t("end")];
    let mut losses = Losses::none();
    let rt = flatten_inlines(&inlines, &mut losses);

    assert_eq!(rt.text, "startend");
    assert!(
        rt.facets.is_empty(),
        "Empty link text should not produce a facet"
    );
}

#[test]
fn zero_length_code_inline_omitted() {
    // Empty code inline should not produce a facet
    let inlines = vec![t("x"), ci(""), t("y")];
    let mut losses = Losses::none();
    let rt = flatten_inlines(&inlines, &mut losses);

    assert_eq!(rt.text, "xy");
    assert!(
        rt.facets.is_empty(),
        "Empty code inline should not produce a facet"
    );
}

// ---------------------------------------------------------------------------
// AC11: Unsupported inline type records a loss and extracts plain text
// ---------------------------------------------------------------------------

#[test]
fn unsupported_inline_records_loss() {
    // QuoteInline is not a supported inline type for AT Protocol facets
    let inlines = vec![t("before "), qi([t("quoted")]), t(" after")];
    let mut losses = Losses::none();
    let rt = flatten_inlines(&inlines, &mut losses);

    // Text should still be extracted
    assert_eq!(rt.text, "before quoted after");
    // No facet emitted for the unsupported type
    assert!(
        rt.facets.is_empty(),
        "Unsupported inline should not produce a facet"
    );
    // A loss should have been recorded
    assert!(
        !losses.is_empty(),
        "Unsupported inline should record a loss"
    );
}

// ---------------------------------------------------------------------------
// AC12: RichText::to_value serializes correctly
// ---------------------------------------------------------------------------

#[test]
fn richtext_to_value_plain_text() {
    let inlines = vec![t("simple text")];
    let mut losses = Losses::none();
    let rt = flatten_inlines(&inlines, &mut losses);
    let value = rt.to_value();

    assert!(value.is_object());
    let obj = value.as_object().expect("Should be an object");
    assert_eq!(
        obj.get("text").and_then(|v| v.as_str()),
        Some("simple text")
    );
    // No facets or empty facets array
    let facets = obj.get("facets").and_then(|v| v.as_array());
    assert!(
        facets.is_none() || facets.is_some_and(|a| a.is_empty()),
        "Plain text should have no facets"
    );
}

#[test]
fn richtext_to_value_with_facets() {
    let inlines = vec![em([t("italic")])];
    let mut losses = Losses::none();
    let rt = flatten_inlines(&inlines, &mut losses);
    let value = rt.to_value();

    let obj = value.as_object().expect("Should be an object");
    assert_eq!(obj.get("text").and_then(|v| v.as_str()), Some("italic"));

    let facets = obj
        .get("facets")
        .and_then(|v| v.as_array())
        .expect("Should have facets array");
    assert_eq!(facets.len(), 1);

    let facet = &facets[0];
    // Verify index has byteStart and byteEnd
    let index = facet.get("index").expect("Facet should have index");
    assert_eq!(index.get("byteStart").and_then(|v| v.as_u64()), Some(0));
    assert_eq!(index.get("byteEnd").and_then(|v| v.as_u64()), Some(6));

    // Verify features array
    let features = facet
        .get("features")
        .and_then(|v| v.as_array())
        .expect("Facet should have features array");
    assert_eq!(features.len(), 2);

    // Each feature should have a $type
    for feature in features {
        assert!(
            feature.get("$type").is_some(),
            "Each feature should have a $type field"
        );
    }
}
