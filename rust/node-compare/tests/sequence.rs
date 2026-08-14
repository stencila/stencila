//! Tests of ordered sequence alignment for repeated properties

use eyre::{Result, bail};
use pretty_assertions::assert_eq;

use stencila_node_compare::{
    Alignment, CompareError, CompareOptions, Correspondence, MatchRule, UnmatchedReason, align,
    align_with_options,
};
use stencila_node_path::{NodePath, NodeSlot};
use stencila_node_type::{NodeProperty, NodeType};
use stencila_schema::{
    Block, Paragraph,
    shortcuts::{art, h1, p, sec, t},
};

/// The path of the item at an index of an article's content
fn content(index: usize) -> NodePath {
    NodePath::from([
        NodeSlot::Property(NodeProperty::Content),
        NodeSlot::Index(index),
    ])
}

/// The pairs of an alignment, as left and right content indices
fn content_pairs(alignment: &Alignment) -> Vec<(usize, usize)> {
    let index = |path: &NodePath| match (path.front(), path.get(1)) {
        (Some(NodeSlot::Property(NodeProperty::Content)), Some(NodeSlot::Index(index)))
            if path.len() == 2 =>
        {
            Some(*index)
        }
        _ => None,
    };

    alignment
        .pairs()
        .filter_map(|(left, right, ..)| Some((index(&left.path)?, index(&right.path)?)))
        .collect()
}

/// A paragraph with the given text and explicit id
fn identified(id: &str, text: &str) -> Block {
    Block::Paragraph(Paragraph {
        id: Some(id.to_string()),
        ..Paragraph::new(vec![t(text)])
    })
}

/// An insertion at the start does not shift every following match
#[test]
fn insertion_at_the_start() -> Result<()> {
    let left = art([p([t("One")]), p([t("Two")])]);
    let right = art([p([t("Zero")]), p([t("One")]), p([t("Two")])]);

    let alignment = align(&left, &right)?;
    assert_eq!(content_pairs(&alignment), vec![(0, 1), (1, 2)]);

    Ok(())
}

/// An insertion in the middle does not cascade through the following items
#[test]
fn insertion_in_the_middle() -> Result<()> {
    let left = art([p([t("One")]), p([t("Three")])]);
    let right = art([p([t("One")]), p([t("Two")]), p([t("Three")])]);

    let alignment = align(&left, &right)?;
    assert_eq!(content_pairs(&alignment), vec![(0, 0), (1, 2)]);

    Ok(())
}

/// An insertion at the end leaves the preceding items paired
#[test]
fn insertion_at_the_end() -> Result<()> {
    let left = art([p([t("One")]), p([t("Two")])]);
    let right = art([p([t("One")]), p([t("Two")]), p([t("Three")])]);

    let alignment = align(&left, &right)?;
    assert_eq!(content_pairs(&alignment), vec![(0, 0), (1, 1)]);

    Ok(())
}

/// A deletion in the middle does not cascade either
#[test]
fn deletion_in_the_middle() -> Result<()> {
    let left = art([p([t("One")]), p([t("Two")]), p([t("Three")])]);
    let right = art([p([t("One")]), p([t("Three")])]);

    let alignment = align(&left, &right)?;
    assert_eq!(content_pairs(&alignment), vec![(0, 0), (2, 1)]);

    Ok(())
}

/// An edited item still pairs, because it is more than half similar
#[test]
fn an_edited_item_still_pairs() -> Result<()> {
    let left = art([p([t("The quick brown fox jumps over the lazy dog")])]);
    let right = art([p([t("The quick brown foxes jump over the lazy dogs")])]);

    let alignment = align(&left, &right)?;
    assert_eq!(content_pairs(&alignment), vec![(0, 0)]);

    Ok(())
}

/// An implausible candidate is left as two gaps, because the gaps cost less
#[test]
fn an_implausible_candidate_is_left_as_two_gaps() -> Result<()> {
    let left = art([p([t("Alpha beta gamma delta epsilon")])]);
    let right = art([p([t("Zulu yankee xray whiskey victor")])]);

    let alignment = align(&left, &right)?;
    assert_eq!(content_pairs(&alignment), Vec::new());

    // The reason distinguishes a refused candidate from no candidate at all
    let Some(Correspondence::LeftOnly { reason, .. }) = alignment
        .correspondences
        .iter()
        .find(|correspondence| correspondence.left().map(|node| &node.path) == Some(&content(0)))
    else {
        bail!("Expected the left paragraph to be left-only")
    };
    assert_eq!(reason, &UnmatchedReason::GapCheaperThanPair);

    Ok(())
}

/// A unique explicit id anchors a pair, even across a reordering
#[test]
fn a_unique_id_anchors() -> Result<()> {
    let left = art([
        identified("alpha", "Alpha beta gamma delta"),
        identified("beta", "Zulu yankee xray whiskey"),
    ]);
    let right = art([
        identified("beta", "Zulu yankee xray whiskey"),
        identified("alpha", "Completely different words entirely"),
    ]);

    let alignment = align(&left, &right)?;

    // `alpha` pairs with `alpha` despite its content having been replaced, because a
    // unique explicit id is a compulsory anchor
    let alpha = alignment
        .pairs()
        .find(|(left, ..)| left.path == content(0))
        .map(|(.., right, info)| (right.path.clone(), info.rule));
    assert_eq!(alpha, Some((content(1), MatchRule::UniqueId)));

    Ok(())
}

/// Duplicate ids are ordinary candidates, not anchors
#[test]
fn duplicate_ids_do_not_anchor() -> Result<()> {
    let left = art([
        identified("same", "Alpha beta gamma"),
        identified("same", "Delta epsilon zeta"),
    ]);
    let right = art([
        identified("same", "Alpha beta gamma"),
        identified("same", "Delta epsilon zeta"),
    ]);

    let alignment = align(&left, &right)?;
    assert_eq!(content_pairs(&alignment), vec![(0, 0), (1, 1)]);

    // Nothing was anchored by id, because the id occurs twice on each side
    assert!(
        !alignment
            .pairs()
            .any(|(.., info)| info.rule == MatchRule::UniqueId)
    );

    Ok(())
}

/// A unique exact subtree among repeated similar subtrees anchors
#[test]
fn a_unique_exact_subtree_anchors() -> Result<()> {
    let left = art([
        p([t("Repeated boilerplate")]),
        p([t("A distinctive and unique sentence")]),
        p([t("Repeated boilerplate")]),
    ]);
    let right = art([
        p([t("Repeated boilerplate")]),
        p([t("Repeated boilerplate")]),
        p([t("A distinctive and unique sentence")]),
    ]);

    let alignment = align(&left, &right)?;

    let distinctive = alignment
        .pairs()
        .find(|(left, ..)| left.path == content(1))
        .map(|(.., right, info)| (right.path.clone(), info.rule));
    assert_eq!(
        distinctive,
        Some((content(2), MatchRule::VerifiedExactFingerprint))
    );

    Ok(())
}

/// A cross-type pair is allowed within the same union slot, and only there
#[test]
fn a_same_slot_cross_type_pair() -> Result<()> {
    let left = art([p([t("Identical inline content here")])]);
    let right = art([h1([t("Identical inline content here")])]);

    let alignment = align(&left, &right)?;

    let pair = alignment
        .pairs()
        .find(|(left, ..)| left.path == content(0))
        .map(|(left, right, ..)| (left.node_type, right.node_type));
    assert_eq!(pair, Some((NodeType::Paragraph, NodeType::Heading)));

    Ok(())
}

/// A cross-type candidate whose content also differs is refused
#[test]
fn a_cross_type_candidate_with_different_content_is_refused() -> Result<()> {
    let left = art([p([t("Alpha beta gamma delta epsilon")])]);
    let right = art([h1([t("Zulu yankee xray whiskey victor")])]);

    let alignment = align(&left, &right)?;
    assert_eq!(content_pairs(&alignment), Vec::new());

    Ok(())
}

/// Crossing anchors resolve to a deterministic maximum non-crossing subset
#[test]
fn crossing_anchors_resolve_deterministically() -> Result<()> {
    let left = art([
        p([t("First distinctive paragraph")]),
        p([t("Second distinctive paragraph")]),
        p([t("Third distinctive paragraph")]),
    ]);
    let right = art([
        p([t("Third distinctive paragraph")]),
        p([t("Second distinctive paragraph")]),
        p([t("First distinctive paragraph")]),
    ]);

    let first = align(&left, &right)?;
    let again = align(&left, &right)?;
    assert_eq!(first, again, "repeated runs differ");

    // Only a non-crossing subset can be aligned in order; the rest are gaps until
    // reorder reconciliation takes them up
    let pairs = content_pairs(&first);
    assert!(!pairs.is_empty());
    assert!(pairs.windows(2).all(|pair| pair[0].1 < pair[1].1));

    Ok(())
}

/// Swapping the inputs and inverting the result yields the same artifact, including
/// for the ambiguous cases where tie-breaking decides
#[test]
fn tie_breaking_survives_inversion() -> Result<()> {
    for (left, right) in [
        (
            art([p([t("Alpha")]), p([t("Beta")])]),
            art([p([t("Gamma")]), p([t("Delta")])]),
        ),
        (
            art([p([t("Repeated")]), p([t("Repeated")]), p([t("Repeated")])]),
            art([p([t("Repeated")]), p([t("Repeated")])]),
        ),
        (
            art([
                p([t("First distinctive paragraph")]),
                p([t("Second distinctive paragraph")]),
                p([t("Third distinctive paragraph")]),
            ]),
            art([
                p([t("Third distinctive paragraph")]),
                p([t("Second distinctive paragraph")]),
                p([t("First distinctive paragraph")]),
            ]),
        ),
        (
            art([sec([p([t("Nested")])]), p([t("Loose")])]),
            art([p([t("Loose")]), sec([p([t("Nested")])])]),
        ),
    ] {
        let forward = align(&left, &right)?;
        let inverted = align(&right, &left)?.invert();
        assert_eq!(forward, inverted);
    }

    Ok(())
}

/// Exceeding the candidate-cell budget returns a typed error, and no artifact
#[test]
fn budget_exhaustion_is_an_error() -> Result<()> {
    let left = art((0..40)
        .map(|index| p([t(format!("Left paragraph number {index}"))]))
        .collect::<Vec<_>>());
    let right = art((0..40)
        .map(|index| p([t(format!("Right paragraph number {index}"))]))
        .collect::<Vec<_>>());

    let options = CompareOptions {
        alignment_cell_budget: 100,
    };

    let Err(CompareError::BudgetExhausted {
        required, allowed, ..
    }) = align_with_options(&left, &right, &options)
    else {
        bail!("Expected the budget to be exhausted")
    };
    assert!(required > allowed);
    assert_eq!(allowed, 100);

    // The same inputs succeed with the default budget
    align(&left, &right)?;

    Ok(())
}

/// Anchors partition the sequences, so an anchored collection costs far fewer cells
#[test]
fn anchors_keep_the_budget_small() -> Result<()> {
    let blocks: Vec<Block> = (0..40)
        .map(|index| p([t(format!("Distinctive paragraph number {index}"))]))
        .collect();
    let left = art(blocks.clone());
    let right = art(blocks);

    // Every item is an exact anchor, so no dynamic programming is needed at all
    let options = CompareOptions {
        alignment_cell_budget: 1,
    };
    let alignment = align_with_options(&left, &right, &options)?;
    assert!(!alignment.has_one_sided());

    Ok(())
}

/// A node compared with itself is difference free and path identical
#[test]
fn self_comparison_is_path_identical() -> Result<()> {
    let node = art([
        sec([p([t("One")]), p([t("Two")])]),
        p([t("Three")]),
        h1([t("Four")]),
    ]);

    let alignment = align(&node, &node)?;
    assert!(!alignment.has_one_sided());
    for (left, right, ..) in alignment.pairs() {
        assert_eq!(left.path, right.path);
    }

    Ok(())
}
