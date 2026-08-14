//! Tests of the alignment artifact and of deterministic correspondence

use std::collections::HashSet;

use eyre::{Result, bail};
use pretty_assertions::assert_eq;

use stencila_node_compare::{
    Alignment, AlignmentFormatVersion, Correspondence, MatchRule, NodeRef, Side, align,
    projection::{Projection, Root},
};
use stencila_node_path::{NodePath, NodeSlot};
use stencila_node_type::{NodeProperty, NodeType};
use stencila_schema::{
    Article, Node, Paragraph, Section,
    shortcuts::{art, p, sec, t},
};

/// A path from a sequence of slots
fn path<const N: usize>(slots: [NodeSlot; N]) -> NodePath {
    NodePath::from(slots)
}

/// The property slot for a node property
fn property(property: NodeProperty) -> NodeSlot {
    NodeSlot::Property(property)
}

/// The correspondence whose left path is the given one
fn by_left<'alignment>(
    alignment: &'alignment Alignment,
    path: &NodePath,
) -> Option<&'alignment Correspondence> {
    alignment
        .correspondences
        .iter()
        .find(|correspondence| correspondence.left().map(|node| &node.path) == Some(path))
}

/// The number of structured occurrences in a node
fn occurrence_count(node: &Node) -> Result<usize> {
    Ok(Projection::new(node, Side::Left)?.occurrences().len())
}

/// The two caller-selected roots always receive a deterministic root correspondence
#[test]
fn roots_always_pair() -> Result<()> {
    let alignment = align(&art([p([t("Hello")])]), &art([p([t("Goodbye")])]))?;

    let root = by_left(&alignment, &NodePath::new());
    let Some(Correspondence::Paired {
        left,
        right,
        match_info,
    }) = root
    else {
        bail!("Expected the roots to be paired")
    };
    assert_eq!(left.path, NodePath::new());
    assert_eq!(right.path, NodePath::new());
    assert_eq!(left.node_type, NodeType::Article);
    assert_eq!(match_info.rule, MatchRule::Root);

    Ok(())
}

/// Every structured occurrence on both sides appears exactly once, and none is paired
/// more than once
#[test]
fn coverage_is_complete_and_single() -> Result<()> {
    let left = art([sec([p([t("One")])]), p([t("Two")])]);
    let right = art([sec([p([t("One")])]), p([t("Two")]), p([t("Three")])]);

    let alignment = align(&left, &right)?;

    let mut left_paths = HashSet::new();
    let mut right_paths = HashSet::new();
    for correspondence in &alignment.correspondences {
        if let Some(node) = correspondence.left() {
            assert!(
                left_paths.insert(node.path.clone()),
                "left occurrence at `{}` appears more than once",
                node.path
            );
        }
        if let Some(node) = correspondence.right() {
            assert!(
                right_paths.insert(node.path.clone()),
                "right occurrence at `{}` appears more than once",
                node.path
            );
        }
    }

    // Every occurrence plus, for each root, the root itself
    assert_eq!(left_paths.len(), occurrence_count(&left)?);
    assert_eq!(right_paths.len(), occurrence_count(&right)?);

    Ok(())
}

/// Exactly equal trees pair completely, with nothing one-sided
#[test]
fn equal_trees_pair_completely() -> Result<()> {
    let node = art([sec([p([t("One")]), p([t("Two")])])]);

    let alignment = align(&node, &node)?;
    assert!(!alignment.has_one_sided());

    // Paired occurrences are path identical
    for (left, right, ..) in alignment.pairs() {
        assert_eq!(left.path, right.path);
        assert_eq!(left.node_type, right.node_type);
    }

    Ok(())
}

/// A structured value in the same singular property of paired parents pairs, even
/// when its type or contents differ
#[test]
fn singular_properties_pair_deterministically() -> Result<()> {
    // `ListItem.item` is an optional singular structured property
    let left = Node::ListItem(stencila_schema::ListItem {
        item: Some(Box::new(Node::Paragraph(Paragraph::new(vec![t("One")])))),
        ..stencila_schema::ListItem::new(Vec::new())
    });
    let right = Node::ListItem(stencila_schema::ListItem {
        item: Some(Box::new(Node::Section(Section::new(Vec::new())))),
        ..stencila_schema::ListItem::new(Vec::new())
    });

    let alignment = align(&left, &right)?;

    let item = path([property(NodeProperty::Item)]);
    let Some(Correspondence::Paired {
        left,
        right,
        match_info,
    }) = by_left(&alignment, &item)
    else {
        bail!("Expected the singular property values to be paired")
    };
    // A complete replacement is a pair with differences, not a removal plus an addition
    assert_eq!(left.node_type, NodeType::Paragraph);
    assert_eq!(right.node_type, NodeType::Section);
    assert_eq!(match_info.rule, MatchRule::SingularProperty);

    Ok(())
}

/// An optional singular structured value present on only one side is one-sided
#[test]
fn singular_value_on_one_side_only() -> Result<()> {
    let left = Node::ListItem(stencila_schema::ListItem {
        item: Some(Box::new(Node::Paragraph(Paragraph::new(vec![t("One")])))),
        ..stencila_schema::ListItem::new(Vec::new())
    });
    let right = Node::ListItem(stencila_schema::ListItem::new(Vec::new()));

    let alignment = align(&left, &right)?;

    let item = path([property(NodeProperty::Item)]);
    let Some(Correspondence::LeftOnly {
        left,
        nearest_one_sided_ancestor,
        ..
    }) = by_left(&alignment, &item)
    else {
        bail!("Expected the singular property value to be left-only")
    };
    assert_eq!(left.node_type, NodeType::Paragraph);
    // It is the root of its one-sided subtree
    assert_eq!(nearest_one_sided_ancestor, &None);

    Ok(())
}

/// A one-sided subtree emits a record for every structured descendant, each naming its
/// nearest one-sided ancestor
#[test]
fn one_sided_subtrees_are_exhaustive() -> Result<()> {
    let left = art([sec([p([t("One")])])]);
    let right = art([]);

    let alignment = align(&left, &right)?;

    // The section, the paragraph and the text are all recorded
    let section = path([property(NodeProperty::Content), NodeSlot::Index(0)]);
    let paragraph = path([
        property(NodeProperty::Content),
        NodeSlot::Index(0),
        property(NodeProperty::Content),
        NodeSlot::Index(0),
    ]);
    let text = path([
        property(NodeProperty::Content),
        NodeSlot::Index(0),
        property(NodeProperty::Content),
        NodeSlot::Index(0),
        property(NodeProperty::Content),
        NodeSlot::Index(0),
    ]);

    let Some(Correspondence::LeftOnly {
        nearest_one_sided_ancestor: section_ancestor,
        ..
    }) = by_left(&alignment, &section)
    else {
        bail!("Expected the section to be left-only")
    };
    assert_eq!(section_ancestor, &None, "the subtree root has no ancestor");

    let Some(Correspondence::LeftOnly {
        nearest_one_sided_ancestor: paragraph_ancestor,
        ..
    }) = by_left(&alignment, &paragraph)
    else {
        bail!("Expected the paragraph to be left-only")
    };
    assert_eq!(
        paragraph_ancestor,
        &Some(NodeRef::new(section.clone(), NodeType::Section)),
        "the nearest one-sided ancestor is the section"
    );

    let Some(Correspondence::LeftOnly {
        nearest_one_sided_ancestor: text_ancestor,
        ..
    }) = by_left(&alignment, &text)
    else {
        bail!("Expected the text to be left-only")
    };
    assert_eq!(
        text_ancestor,
        &Some(NodeRef::new(paragraph, NodeType::Paragraph)),
        "the nearest one-sided ancestor is the paragraph, not the subtree root"
    );

    Ok(())
}

/// Occurrences are referenced by path and node type, and no internal UID appears
#[test]
fn references_resolve_in_their_projection() -> Result<()> {
    let left = art([sec([p([t("One")])])]);
    let right = art([p([t("Two")])]);

    let alignment = align(&left, &right)?;

    let left_projection = Projection::new(&left, Side::Left)?;
    let right_projection = Projection::new(&right, Side::Right)?;

    for correspondence in &alignment.correspondences {
        for (node, projection) in [
            (correspondence.left(), &left_projection),
            (correspondence.right(), &right_projection),
        ] {
            let Some(node) = node else { continue };

            // The root reference may be a scalar root; here both roots are structured
            let resolved = projection
                .occurrences()
                .iter()
                .find(|occurrence| occurrence.path == node.path);
            let Some(resolved) = resolved else {
                bail!("Reference to `{}` does not resolve", node.path)
            };
            assert_eq!(resolved.node_type, node.node_type);
        }
    }

    // No UID appears anywhere in the serialized artifact
    let json = serde_json::to_string(&alignment)?;
    assert!(!json.contains("uid"));

    Ok(())
}

/// Correspondences are canonically ordered in memory and after deserialization
#[test]
fn canonical_ordering_holds() -> Result<()> {
    let alignment = align(
        &art([sec([p([t("One")])]), p([t("Two")])]),
        &art([p([t("Two")])]),
    )?;

    let ordered = |alignment: &Alignment| alignment.correspondences.is_sorted();
    assert!(ordered(&alignment), "not ordered in memory");

    let json = serde_json::to_string(&alignment)?;
    let round_tripped: Alignment = serde_json::from_str(&json)?;
    assert!(ordered(&round_tripped), "not ordered after deserialization");
    assert_eq!(round_tripped, alignment);

    Ok(())
}

/// Swapping the inputs and inverting the result yields the same canonical artifact
#[test]
fn swap_and_invert_is_identical() -> Result<()> {
    let left = art([sec([p([t("One")])]), p([t("Two")])]);
    let right = art([p([t("Two")]), p([t("Three")]), sec([])]);

    let forward = align(&left, &right)?;
    let inverted = align(&right, &left)?.invert();

    assert_eq!(forward, inverted);

    Ok(())
}

/// The artifact is versioned with algorithm, projection and policy identifiers
#[test]
fn artifact_is_versioned() -> Result<()> {
    let alignment = align(&art([]), &art([]))?;

    assert_eq!(alignment.format_version, AlignmentFormatVersion::V1);
    assert_eq!(alignment.algorithm.name, "stencila-schema-native");
    assert_eq!(alignment.algorithm.version, "1");
    assert_eq!(alignment.algorithm.projection_version, "1");
    assert_eq!(alignment.algorithm.policy, "schema-native");

    Ok(())
}

/// The rejected candidate matrix is not retained
#[test]
fn rejected_candidates_are_not_retained() -> Result<()> {
    let left = art([p([t("One")]), p([t("Two")]), p([t("Three")])]);
    let right = art([p([t("Four")]), p([t("Five")]), p([t("Six")])]);

    let alignment = align(&left, &right)?;

    // Evidence explains the selected outcome only, so it is bounded by the number of
    // signals rather than by the number of candidates
    for (.., match_info) in alignment.pairs() {
        assert!(match_info.evidence.len() <= 8);
    }

    Ok(())
}

/// Paired records carry a rule, a pair cost and gap costs
#[test]
fn pairs_are_explainable() -> Result<()> {
    let alignment = align(&art([p([t("One")])]), &art([p([t("One")])]))?;

    for (left, .., match_info) in alignment.pairs() {
        assert!(
            match_info.left_gap_cost.units() > 0 || left.path.is_empty(),
            "a gap cost should be recorded for `{}`",
            left.path
        );
    }

    Ok(())
}

/// A scalar root pairs with a structured root, without its contents being forced into
/// a structural comparison
#[test]
fn scalar_and_structured_roots_pair() -> Result<()> {
    let left = Node::Integer(42);
    let right = art([p([t("One")])]);

    let alignment = align(&left, &right)?;

    let Some(Correspondence::Paired {
        left: left_ref,
        right: right_ref,
        ..
    }) = alignment.correspondences.iter().find(|correspondence| {
        matches!(correspondence, Correspondence::Paired { left, .. } if left.path.is_empty())
    }) else {
        bail!("Expected the roots to be paired")
    };
    assert_eq!(left_ref.node_type, NodeType::Integer);
    assert_eq!(right_ref.node_type, NodeType::Article);

    // The article's descendants have no counterpart
    assert!(alignment.has_one_sided());

    Ok(())
}

/// Two scalar roots pair and add no other correspondence
#[test]
fn scalar_roots_pair() -> Result<()> {
    let alignment = align(&Node::Integer(1), &Node::String("one".to_string()))?;

    assert_eq!(alignment.correspondences.len(), 1);
    let Some(Correspondence::Paired { left, right, .. }) = alignment.correspondences.first() else {
        bail!("Expected the roots to be paired")
    };
    assert_eq!(left.node_type, NodeType::Integer);
    assert_eq!(right.node_type, NodeType::String);

    Ok(())
}

/// An article's root is structured, so an alignment of two articles has one
/// correspondence per occurrence
#[test]
fn occurrence_counts_match() -> Result<()> {
    let left = art([p([t("One")])]);
    let right = art([p([t("One")])]);

    let alignment = align(&left, &right)?;
    let Root::Structured(..) = Projection::new(&left, Side::Left)?.root() else {
        bail!("Expected a structured root")
    };

    assert_eq!(alignment.correspondences.len(), occurrence_count(&left)?);
    assert_eq!(
        Article::default().id,
        None,
        "the fixture relies on articles having no explicit id"
    );

    Ok(())
}
