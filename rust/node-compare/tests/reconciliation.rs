//! Tests of within-scope and cross-parent reconciliation

use eyre::{Result, bail};
use pretty_assertions::assert_eq;

use stencila_node_compare::{Alignment, Difference, MatchRule, align, compare};
use stencila_node_path::{NodePath, NodeSlot};
use stencila_node_type::{NodeProperty, NodeType};
use stencila_schema::{
    Block, Emphasis, Inline, Paragraph, Section,
    shortcuts::{art, p, sec, t},
};

/// A path built from slots
fn path<const N: usize>(slots: [NodeSlot; N]) -> NodePath {
    NodePath::from(slots)
}

/// A paragraph with an explicit id
fn identified(id: &str, text: &str) -> Block {
    Block::Paragraph(Paragraph {
        id: Some(id.to_string()),
        ..Paragraph::new(vec![t(text)])
    })
}

/// A section with an explicit id
fn identified_section(id: &str, content: Vec<Block>) -> Block {
    Block::Section(Section {
        id: Some(id.to_string()),
        ..Section::new(content)
    })
}

/// The right path a left path is paired with, and the rule that selected the pair
fn pairing(alignment: &Alignment, left: &NodePath) -> Option<(NodePath, MatchRule)> {
    alignment
        .pairs()
        .find(|(candidate, ..)| &candidate.path == left)
        .map(|(.., right, info)| (right.path.clone(), info.rule))
}

/// A subtree that moved to a different parent is recognised by its unique explicit id
#[test]
fn a_cross_parent_move_by_unique_id() -> Result<()> {
    let (left, right) = moved_between_sections();

    let alignment = align(&left, &right)?;

    let from = path([
        NodeSlot::Property(NodeProperty::Content),
        NodeSlot::Index(0),
        NodeSlot::Property(NodeProperty::Content),
        NodeSlot::Index(0),
    ]);
    let (to, rule) = pairing(&alignment, &from)
        .ok_or_else(|| eyre::eyre!("Expected the moved paragraph to be paired"))?;
    assert_eq!(
        to,
        path([
            NodeSlot::Property(NodeProperty::Content),
            NodeSlot::Index(1),
            NodeSlot::Property(NodeProperty::Content),
            NodeSlot::Index(1)
        ])
    );
    assert_eq!(rule, MatchRule::CrossParentReconciliation);

    Ok(())
}

/// Two identified sections, with one identified paragraph moving from the first to
/// the second
///
/// The sections are anchored by their own ids, so that they pair with each other and
/// the paragraph is left as the only thing that could have moved.
fn moved_between_sections() -> (stencila_schema::Node, stencila_schema::Node) {
    let left = art([
        identified_section(
            "first",
            vec![identified("moved", "The paragraph that moves house")],
        ),
        identified_section("second", vec![p([t("The paragraph that stays put")])]),
    ]);
    let right = art([
        identified_section("first", Vec::new()),
        identified_section(
            "second",
            vec![
                p([t("The paragraph that stays put")]),
                identified("moved", "The paragraph that moves house"),
            ],
        ),
    ]);

    (left, right)
}

/// An identity-neutral fingerprint recognises a move even when the id was edited
#[test]
fn a_cross_parent_move_by_identity_neutral_equality() -> Result<()> {
    let moved = |id: &str| {
        identified_section(
            id,
            vec![p([t("A distinctive and quite unmistakable paragraph")])],
        )
    };

    let left = art([
        identified_section("first", vec![moved("before")]),
        identified_section("second", vec![p([t("The paragraph that stays put")])]),
    ]);
    let right = art([
        identified_section("first", Vec::new()),
        identified_section(
            "second",
            vec![p([t("The paragraph that stays put")]), moved("after")],
        ),
    ]);

    let alignment = align(&left, &right)?;

    let from = path([
        NodeSlot::Property(NodeProperty::Content),
        NodeSlot::Index(0),
        NodeSlot::Property(NodeProperty::Content),
        NodeSlot::Index(0),
    ]);
    let (to, rule) = pairing(&alignment, &from)
        .ok_or_else(|| eyre::eyre!("Expected the moved section to be paired"))?;
    assert_eq!(
        to,
        path([
            NodeSlot::Property(NodeProperty::Content),
            NodeSlot::Index(1),
            NodeSlot::Property(NodeProperty::Content),
            NodeSlot::Index(1)
        ])
    );
    assert_eq!(rule, MatchRule::CrossParentReconciliation);

    Ok(())
}

/// A modified, id-less cross-parent candidate is deliberately left unmatched
#[test]
fn a_modified_id_less_move_is_left_unmatched() -> Result<()> {
    let left = art([
        sec([p([t("A distinctive and quite unmistakable paragraph")])]),
        sec([]),
    ]);
    let right = art([
        sec([]),
        sec([p([t(
            "A distinctive and quite unmistakable paragraph, revised",
        )])]),
    ]);

    let alignment = align(&left, &right)?;

    // No fuzzy text similarity is used across parents, so the two remain one-sided
    let from = path([
        NodeSlot::Property(NodeProperty::Content),
        NodeSlot::Index(0),
        NodeSlot::Property(NodeProperty::Content),
        NodeSlot::Index(0),
    ]);
    assert_eq!(pairing(&alignment, &from), None);
    assert!(alignment.has_one_sided());

    Ok(())
}

/// Duplicate identities do not produce cross-parent pairs
#[test]
fn duplicates_do_not_move() -> Result<()> {
    let left = art([
        sec([
            identified("same", "Repeated"),
            identified("same", "Repeated"),
        ]),
        sec([]),
    ]);
    let right = art([
        sec([]),
        sec([
            identified("same", "Repeated"),
            identified("same", "Repeated"),
        ]),
    ]);

    let alignment = align(&left, &right)?;

    assert!(
        !alignment
            .pairs()
            .any(|(.., info)| info.rule == MatchRule::CrossParentReconciliation),
        "duplicate ids and duplicate equal subtrees are not strong evidence"
    );

    Ok(())
}

/// A strongly identified child can move out of a removed container
#[test]
fn a_child_moves_out_of_a_removed_container() -> Result<()> {
    // The section is removed, but the paragraph it contained survives at the top level
    let left = art([sec([
        p([t("Filler that makes the section quite unlike its child")]),
        identified("survivor", "Short"),
    ])]);
    let right = art([identified("survivor", "Short")]);

    let alignment = align(&left, &right)?;

    let from = path([
        NodeSlot::Property(NodeProperty::Content),
        NodeSlot::Index(0),
        NodeSlot::Property(NodeProperty::Content),
        NodeSlot::Index(1),
    ]);
    let (to, rule) = pairing(&alignment, &from)
        .ok_or_else(|| eyre::eyre!("Expected the surviving paragraph to be paired"))?;
    assert_eq!(
        to,
        path([
            NodeSlot::Property(NodeProperty::Content),
            NodeSlot::Index(0)
        ])
    );
    assert_eq!(rule, MatchRule::CrossParentReconciliation);

    Ok(())
}

/// A move is reported as a parent change
#[test]
fn a_move_is_a_parent_change() -> Result<()> {
    let (left, right) = moved_between_sections();

    let comparison = compare(&left, &right)?;

    let changes: Vec<&Difference> = comparison
        .differences()
        .iter()
        .filter(|difference| matches!(difference, Difference::ParentChanged { .. }))
        .collect();
    assert_eq!(changes.len(), 1);
    let Some(Difference::ParentChanged {
        left_parent,
        right_parent,
        left_property,
        right_property,
        ..
    }) = changes.first()
    else {
        bail!("Expected a parent change")
    };
    assert_eq!(
        left_parent.as_ref().map(|node| node.node_type),
        Some(NodeType::Section)
    );
    assert_eq!(
        right_parent.as_ref().map(|node| node.node_type),
        Some(NodeType::Section)
    );
    assert_ne!(
        left_parent.as_ref().map(|node| &node.path),
        right_parent.as_ref().map(|node| &node.path)
    );
    assert_eq!(left_property, &Some(NodeProperty::Content));
    assert_eq!(right_property, &Some(NodeProperty::Content));

    Ok(())
}

/// Movement is never inferred from index inequality alone
#[test]
fn an_index_shift_is_not_a_parent_change() -> Result<()> {
    let left = art([p([t("One")]), p([t("Two")])]);
    let right = art([p([t("Zero")]), p([t("One")]), p([t("Two")])]);

    let comparison = compare(&left, &right)?;

    assert!(
        !comparison
            .differences()
            .iter()
            .any(|difference| matches!(difference, Difference::ParentChanged { .. })),
        "inserting an early sibling is not a move"
    );

    Ok(())
}

/// Complete coverage still holds after reconciliation
#[test]
fn coverage_holds_after_reconciliation() -> Result<()> {
    let (left, right) = moved_between_sections();

    let alignment = align(&left, &right)?;

    let mut left_paths = std::collections::HashSet::new();
    let mut right_paths = std::collections::HashSet::new();
    for correspondence in alignment.correspondences() {
        if let Some(node) = correspondence.left() {
            assert!(left_paths.insert(node.path.clone()));
        }
        if let Some(node) = correspondence.right() {
            assert!(right_paths.insert(node.path.clone()));
        }
    }

    // Swap symmetry survives reconciliation
    assert_eq!(align(&right, &left)?.invert(), alignment);

    Ok(())
}

/// A shared explicit id does not pair two differently typed subtrees across parents
///
/// Within a sibling scope, two differently typed items may still be compatible when
/// the property is declared identically on both sides and holds a union. Across
/// parents there is no such shared declaration to appeal to, so an id alone must not
/// A cross-parent pair may cross type within one union
///
/// Two occurrences in different parents are never weighed against each other by the
/// ordered alignment, so what makes them comparable has to be argued from the schema.
/// The properties holding them are necessarily different, but the *slot* those
/// properties hold can be the same union — a `Section` and a `Paragraph` are both valid
/// wherever blocks are accepted — and that is enough. Requiring the concrete types to
/// agree meant a section flattened into bare paragraphs by another tool went
/// unrecognised.
#[test]
fn a_cross_parent_pair_may_cross_type_within_a_union() -> Result<()> {
    let left = art([
        identified_section("first", vec![identified("shared", "A sentence")]),
        identified_section("second", vec![p([t("Something else entirely")])]),
    ]);
    let right = art([
        identified_section("first", vec![p([t("Something else entirely")])]),
        identified_section(
            "second",
            vec![Block::Section(Section {
                id: Some("shared".to_string()),
                ..Section::new(vec![p([t("A sentence")])])
            })],
        ),
    ]);

    let alignment = align(&left, &right)?;

    // The paragraph and the section share an id, and both sit where blocks are
    // accepted, so the shared identity is honoured across the two parents
    let paragraph = path([
        NodeSlot::Property(NodeProperty::Content),
        NodeSlot::Index(0),
        NodeSlot::Property(NodeProperty::Content),
        NodeSlot::Index(0),
    ]);
    let (to, rule) = pairing(&alignment, &paragraph)
        .ok_or_else(|| eyre::eyre!("Expected the identified paragraph to be paired"))?;
    assert_eq!(rule, MatchRule::CrossParentReconciliation);
    assert_eq!(
        to,
        path([
            NodeSlot::Property(NodeProperty::Content),
            NodeSlot::Index(1),
            NodeSlot::Property(NodeProperty::Content),
            NodeSlot::Index(0)
        ])
    );

    Ok(())
}

/// A cross-parent pair never crosses from one union to another
///
/// Sharing an identity is not enough on its own. A block and an inline cannot stand in
/// for one another anywhere in the schema, so pairing them would describe a change that
/// no edit could have made.
#[test]
fn a_cross_parent_pair_never_crosses_unions() -> Result<()> {
    let left = art([
        identified_section("first", vec![identified("shared", "A sentence")]),
        identified_section("second", vec![p([t("Something else entirely")])]),
    ]);
    let right = art([
        identified_section("first", vec![p([t("Something else entirely")])]),
        identified_section(
            "second",
            vec![Block::Paragraph(Paragraph::new(vec![Inline::Emphasis(
                Emphasis {
                    id: Some("shared".to_string()),
                    ..Emphasis::new(vec![t("A sentence")])
                },
            )]))],
        ),
    ]);

    let alignment = align(&left, &right)?;

    for (left, right, ..) in alignment.pairs() {
        if left.node_type == NodeType::Paragraph && right.node_type == NodeType::Emphasis {
            bail!("A cross-parent pair joined a block to an inline")
        }
    }

    Ok(())
}
