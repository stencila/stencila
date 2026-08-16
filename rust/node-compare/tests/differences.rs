//! Tests of property and value differences

use eyre::{Result, bail};
use pretty_assertions::assert_eq;

use stencila_node_compare::{
    Comparison, ComparisonFormatVersion, Difference, PropertyPresence, ScalarValue, ValueState,
    compare,
};
use stencila_node_path::{NodePath, NodeSlot};
use stencila_node_type::{NodeProperty, NodeType};
use stencila_schema::{
    Article, Block, Cord, CordAuthorship, Heading, Node, Paragraph, Text,
    shortcuts::{art, h1, p, sec, t},
};

/// The path of the item at an index of an article's content
fn content(index: usize) -> NodePath {
    NodePath::from([
        NodeSlot::Property(NodeProperty::Content),
        NodeSlot::Index(index),
    ])
}

/// The differences of a given kind
fn of_property(comparison: &Comparison, property: NodeProperty) -> Vec<&Difference> {
    comparison
        .differences()
        .iter()
        .filter(|difference| difference.property() == Some(property))
        .collect()
}

/// An article, or fail
fn article(node: Node) -> Result<Article> {
    match node {
        Node::Article(article) => Ok(article),
        _ => bail!("Expected an article"),
    }
}

/// A changed scalar property produces exactly one difference carrying both values
#[test]
fn a_changed_scalar_is_one_difference() -> Result<()> {
    let comparison = compare(
        &Node::Text(Text::from("Hello")),
        &Node::Text(Text::from("Goodbye")),
    )?;

    let values = of_property(&comparison, NodeProperty::Value);
    assert_eq!(values.len(), 1);
    let Some(Difference::ValueChanged { left, right, .. }) = values.first() else {
        bail!("Expected a value change")
    };
    assert_eq!(
        left,
        &ValueState::One {
            value: ScalarValue::string("Hello")
        }
    );
    assert_eq!(
        right,
        &ValueState::One {
            value: ScalarValue::string("Goodbye")
        }
    );

    Ok(())
}

/// A `Cord` compares by its string only, so an authorship-only change is not a
/// difference
#[test]
fn cord_authorship_is_ignored() -> Result<()> {
    let plain = Text::from("Hello");
    let authored = Text {
        value: Cord {
            string: "Hello".to_string(),
            authorship: vec![CordAuthorship::new(1, 1, 1, 5)],
        },
        ..Text::from("Hello")
    };

    let comparison = compare(&Node::Text(plain.clone()), &Node::Text(authored.clone()))?;
    assert!(comparison.differences().is_empty());
    assert!(comparison.is_equal());

    // A string change is still exact
    let changed = Text::from("Goodbye");
    let comparison = compare(&Node::Text(authored), &Node::Text(changed))?;
    assert_eq!(of_property(&comparison, NodeProperty::Value).len(), 1);

    Ok(())
}

/// `None` and `Some(empty)` are different presences
#[test]
fn none_differs_from_some_empty() -> Result<()> {
    let absent = Paragraph::new(vec![t("Hello")]);
    let empty = Paragraph {
        authors: Some(Vec::new()),
        ..Paragraph::new(vec![t("Hello")])
    };

    let comparison = compare(&Node::Paragraph(absent), &Node::Paragraph(empty))?;

    let authors = of_property(&comparison, NodeProperty::Authors);
    assert_eq!(authors.len(), 1);
    let Some(Difference::PropertyPresenceChanged {
        left_presence,
        right_presence,
        ..
    }) = authors.first()
    else {
        bail!("Expected a presence change")
    };
    assert_eq!(left_presence, &PropertyPresence::Absent);
    assert_eq!(right_presence, &PropertyPresence::Present);

    Ok(())
}

/// An optional scalar records absence versus its typed present value
#[test]
fn optional_scalar_presence_is_a_value_change() -> Result<()> {
    let absent = Heading::new(1, vec![t("Hello")]);
    let present = Heading {
        label: Some("intro".to_string()),
        ..Heading::new(1, vec![t("Hello")])
    };

    let comparison = compare(&Node::Heading(absent), &Node::Heading(present))?;
    let labels = of_property(&comparison, NodeProperty::Label);
    assert_eq!(labels.len(), 1);
    let Some(Difference::ValueChanged { left, right, .. }) = labels.first() else {
        bail!("Expected an optional scalar value change")
    };
    assert_eq!(left, &ValueState::Absent);
    assert_eq!(
        right,
        &ValueState::One {
            value: ScalarValue::string("intro")
        }
    );

    Ok(())
}

/// A cross-type pair emits a node type change, still compares its shared properties,
/// and reports the properties only one of its types declares
#[test]
fn a_node_type_change_with_equal_shared_content() -> Result<()> {
    let left = art([p([t("Identical inline content here")])]);
    let right = art([h1([t("Identical inline content here")])]);

    let comparison = compare(&left, &right)?;

    // The type change is recorded
    let changed: Vec<&Difference> = comparison
        .differences()
        .iter()
        .filter(|difference| matches!(difference, Difference::NodeTypeChanged { .. }))
        .collect();
    assert_eq!(changed.len(), 1);
    assert_eq!(changed[0].left().path, content(0));
    assert_eq!(changed[0].left().node_type, NodeType::Paragraph);
    assert_eq!(changed[0].right().node_type, NodeType::Heading);

    // The shared `content` property was still compared recursively, so its identical
    // text produced no difference at all
    assert!(
        comparison
            .differences()
            .iter()
            .all(|difference| difference.property() != Some(NodeProperty::Value))
    );

    // `level` is declared by `Heading` but not by `Paragraph`
    let level = of_property(&comparison, NodeProperty::Level);
    assert_eq!(level.len(), 1);
    let Some(Difference::PropertyPresenceChanged {
        left_presence,
        right_presence,
        ..
    }) = level.first()
    else {
        bail!("Expected a presence change")
    };
    assert_eq!(left_presence, &PropertyPresence::Undeclared);
    assert_eq!(right_presence, &PropertyPresence::Present);

    Ok(())
}

/// A homogeneous repeated scalar property produces one sequence-valued difference
#[test]
fn a_repeated_scalar_is_one_sequence_difference() -> Result<()> {
    let mut left = article(art([]))?;
    left.options.keywords = Some(vec!["one".to_string(), "two".to_string()]);
    let mut right = article(art([]))?;
    right.options.keywords = Some(vec!["one".to_string(), "three".to_string()]);

    let comparison = compare(&Node::Article(left), &Node::Article(right))?;

    let keywords = of_property(&comparison, NodeProperty::Keywords);
    assert_eq!(keywords.len(), 1, "expected exactly one difference");
    let Some(Difference::ValueChanged { left, right, .. }) = keywords.first() else {
        bail!("Expected a value change")
    };
    assert_eq!(
        left,
        &ValueState::Many {
            values: vec![ScalarValue::string("one"), ScalarValue::string("two")]
        }
    );
    assert_eq!(
        right,
        &ValueState::Many {
            values: vec![ScalarValue::string("one"), ScalarValue::string("three")]
        }
    );

    Ok(())
}

/// A structured property recurses, and does not also emit its whole containing value
#[test]
fn structured_properties_recurse() -> Result<()> {
    let left = art([p([t("One"), t("The quick brown fox")])]);
    let right = art([p([t("One"), t("The quick brown foxes")])]);

    let comparison = compare(&left, &right)?;

    // Exactly one leaf value changed: the paragraph's `content` is not itself recorded
    let values: Vec<&Difference> = comparison
        .differences()
        .iter()
        .filter(|difference| matches!(difference, Difference::ValueChanged { .. }))
        .collect();
    assert_eq!(values.len(), 1);
    assert_eq!(values[0].property(), Some(NodeProperty::Value));

    Ok(())
}

/// One-sided occurrences produce no differences, and no recursive leaf values
#[test]
fn one_sided_occurrences_produce_no_differences() -> Result<()> {
    let left = art([sec([p([t("One")]), p([t("Two")])])]);
    let right = art([]);

    let comparison = compare(&left, &right)?;

    assert!(
        comparison.differences().is_empty(),
        "a missing subtree is captured by the alignment alone"
    );
    assert!(comparison.alignment().has_one_sided());
    assert!(!comparison.is_equal());

    Ok(())
}

/// Equality is no one-sided correspondences and no differences, and matches canonical
/// projection equality exactly
#[test]
fn equality_matches_projection_equality() -> Result<()> {
    let cases: Vec<(Node, Node, bool)> = vec![
        (art([p([t("One")])]), art([p([t("One")])]), true),
        (art([p([t("One")])]), art([p([t("Two")])]), false),
        (art([p([t("One")])]), art([]), false),
        (art([]), art([p([t("One")])]), false),
        (Node::Integer(1), Node::Integer(1), true),
        (Node::Integer(1), Node::Integer(2), false),
        (Node::Integer(1), Node::String("1".to_string()), false),
    ];

    for (left, right, expected) in cases {
        let comparison = compare(&left, &right)?;
        assert_eq!(
            comparison.is_equal(),
            expected,
            "for {left:?} against {right:?}"
        );
    }

    Ok(())
}

/// Two same-type primitive roots compare through a root value location
#[test]
fn primitive_roots_compare_by_value() -> Result<()> {
    let comparison = compare(&Node::Integer(1), &Node::Integer(2))?;

    assert_eq!(comparison.differences().len(), 1);
    let Some(Difference::ValueChanged { location, .. }) = comparison.differences().first() else {
        bail!("Expected a value change")
    };
    assert_eq!(
        location.property, None,
        "the root has no containing property"
    );
    assert!(location.left.path.is_empty());

    Ok(())
}

/// An incompatible root pair records a node type change, without recursing into the
/// contents of either
#[test]
fn incompatible_roots_change_type() -> Result<()> {
    let comparison = compare(&Node::Integer(1), &Node::String("one".to_string()))?;

    assert_eq!(comparison.differences().len(), 1);
    assert!(matches!(
        comparison.differences().first(),
        Some(Difference::NodeTypeChanged { .. })
    ));

    Ok(())
}

/// Differences are canonically ordered in memory and after deserialization, and
/// inversion re-canonicalizes them
#[test]
fn canonical_ordering_and_inversion() -> Result<()> {
    let left = art([p([t("One")]), sec([p([t("Two")])]), h1([t("Three")])]);
    let right = art([p([t("Uno")]), sec([p([t("Two")])]), p([t("Three")])]);

    let comparison = compare(&left, &right)?;
    assert!(
        comparison.differences().is_sorted(),
        "not ordered in memory"
    );

    let json = serde_json::to_string(&comparison)?;
    let round_tripped: Comparison = serde_json::from_str(&json)?;
    assert!(
        round_tripped.differences().is_sorted(),
        "not ordered after deserialization"
    );
    assert_eq!(round_tripped, comparison);

    let inverted = compare(&right, &left)?.invert();
    assert_eq!(inverted, comparison);

    Ok(())
}

/// The comparison is versioned, and carries its alignment
#[test]
fn the_artifact_is_versioned() -> Result<()> {
    let comparison = compare(&art([]), &art([]))?;

    assert_eq!(comparison.format_version(), ComparisonFormatVersion::V2);
    assert_eq!(comparison.algorithm().name, "stencila-schema-native");
    assert_eq!(comparison.algorithm(), comparison.alignment().algorithm());

    Ok(())
}

/// Ordering distinguishes differences whose canonical locations agree but values differ
#[test]
fn ordering_agrees_with_equality() -> Result<()> {
    let one = compare(&Node::Integer(1), &Node::Integer(2))?
        .differences()
        .first()
        .cloned()
        .ok_or_else(|| eyre::eyre!("expected a difference"))?;
    let other = compare(&Node::Integer(1), &Node::Integer(3))?
        .differences()
        .first()
        .cloned()
        .ok_or_else(|| eyre::eyre!("expected a difference"))?;

    assert_ne!(one, other);
    assert_ne!(one.cmp(&other), std::cmp::Ordering::Equal);

    Ok(())
}

/// Matching normalization cannot hide an exact value difference
#[test]
fn normalization_cannot_hide_a_difference() -> Result<()> {
    // The aligner normalizes whitespace in order to find the pair; the value policy
    // still records the original strings as different
    let left: Block = p([t("Hello   world")]);
    let right: Block = p([t("Hello world")]);

    let comparison = compare(&art([left]), &art([right]))?;

    assert!(!comparison.is_equal());
    assert_eq!(of_property(&comparison, NodeProperty::Value).len(), 1);

    Ok(())
}
