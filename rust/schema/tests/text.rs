//! Tests of the `TextCodec` derive
//!
//! The derive picks one field to be a type's text, from two tiers: the type's main
//! content, or failing that the property carrying its identity. These tests pin both the
//! fallback and, more importantly, that it never displaces a type's content.

use stencila_codec_text_trait::to_text;

use stencila_schema::{
    Agent, Block, Date, DateTime, Organization, Paragraph, Periodical, Primitive, PropertyValue,
    shortcuts::t,
};

/// A type whose whole meaning is a `value` reads as that value
#[test]
fn value_bearing_types_have_text() {
    assert_eq!(to_text(&Date::new("2020-05-01".to_string())), "2020-05-01");
    assert_eq!(
        to_text(&DateTime::new("2020-05-01T09:00:00".to_string())),
        "2020-05-01T09:00:00"
    );
    assert_eq!(
        to_text(&PropertyValue::new(Primitive::String("pmid-1".to_string()))),
        "pmid-1"
    );
}

/// A type whose whole meaning is a `name` reads as that name
#[test]
fn name_bearing_types_have_text() {
    assert_eq!(
        to_text(&Organization {
            name: Some("Hebrew University".to_string()),
            ..Default::default()
        }),
        "Hebrew University"
    );
    assert_eq!(
        to_text(&Periodical {
            name: Some("Journal of Things".to_string()),
            ..Default::default()
        }),
        "Journal of Things"
    );
}

/// A type that declares a `name` before its content still reads as its content
///
/// `Agent`, `File`, `Prompt`, `Skill` and `Workflow` all declare `name` first, so the
/// identity tier must never take precedence over the content tier.
#[test]
fn content_wins_over_name() {
    let agent = Agent {
        name: "reviewer".to_string(),
        content: Some(vec![Block::Paragraph(Paragraph::new(vec![t(
            "The instructions",
        )]))]),
        ..Default::default()
    };

    assert_eq!(to_text(&agent).trim(), "The instructions");
}
