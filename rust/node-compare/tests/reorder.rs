//! Tests of reorder observations within aligned sibling scopes

use eyre::{Result, bail};
use pretty_assertions::assert_eq;

use stencila_node_compare::{Comparison, Difference, compare};
use stencila_node_path::{NodePath, NodeSlot};
use stencila_node_type::{NodeProperty, NodeType};
use stencila_schema::{
    Node,
    shortcuts::{art, p, t},
};

/// An article whose content is a paragraph per text
fn article(texts: [&str; 4]) -> Node {
    art(texts.iter().map(|text| p([t(*text)])).collect::<Vec<_>>())
}

/// The path of the item at an index of an article's content
fn content(index: usize) -> NodePath {
    NodePath::from([
        NodeSlot::Property(NodeProperty::Content),
        NodeSlot::Index(index),
    ])
}

/// The reorder observations of a comparison, as pairs of paths
fn reorders(comparison: &Comparison) -> Vec<(NodePath, NodePath)> {
    comparison
        .differences()
        .iter()
        .filter_map(|difference| match difference {
            Difference::Reordered { left, right, .. } => {
                Some((left.path.clone(), right.path.clone()))
            }
            _ => None,
        })
        .collect()
}

/// A sibling that moved relative to the others is reported, and only it
#[test]
fn a_moved_sibling_is_reordered() -> Result<()> {
    // The first paragraph moves to the end; the other three keep their relative order
    let left = article(["one", "two", "three", "four"]);
    let right = article(["two", "three", "four", "one"]);

    let comparison = compare(&left, &right)?;

    assert_eq!(reorders(&comparison), vec![(content(0), content(3))]);

    Ok(())
}

/// A reorder observation references its occurrences and its aligned sibling scope
/// using typed locations only
#[test]
fn a_reorder_locates_its_scope() -> Result<()> {
    let left = article(["one", "two", "three", "four"]);
    let right = article(["two", "three", "four", "one"]);

    let comparison = compare(&left, &right)?;

    let Some(Difference::Reordered {
        left,
        right,
        left_scope,
        right_scope,
        property,
    }) = comparison
        .differences()
        .iter()
        .find(|difference| matches!(difference, Difference::Reordered { .. }))
    else {
        bail!("Expected a reorder observation")
    };

    assert_eq!(left.node_type, NodeType::Paragraph);
    assert_eq!(right.node_type, NodeType::Paragraph);
    assert_eq!(*property, NodeProperty::Content);

    for scope in [left_scope, right_scope] {
        let Some(scope) = scope else {
            bail!("Expected an aligned sibling scope")
        };
        assert_eq!(scope.node_type, NodeType::Article);
        assert_eq!(scope.path, NodePath::default());
    }

    Ok(())
}

/// Inserting an early sibling shifts every later index, and moves nothing
#[test]
fn an_insertion_is_not_a_reorder() -> Result<()> {
    let left = art([p([t("one")]), p([t("two")]), p([t("three")])]);
    let right = article(["zero", "one", "two", "three"]);

    let comparison = compare(&left, &right)?;

    assert_eq!(reorders(&comparison), Vec::new());

    Ok(())
}

/// Deleting an early sibling shifts every later index, and moves nothing
#[test]
fn a_deletion_is_not_a_reorder() -> Result<()> {
    let left = article(["one", "two", "three", "four"]);
    let right = art([p([t("two")]), p([t("three")]), p([t("four")])]);

    let comparison = compare(&left, &right)?;

    assert_eq!(reorders(&comparison), Vec::new());

    Ok(())
}

/// The preserved-order subset is of maximum size, so the output is linear in the
/// number of pairs rather than quadratic in the number of inversions
#[test]
fn output_is_linear_in_the_number_of_pairs() -> Result<()> {
    // Fully reversed: every pair but one is outside any preserved-order subset, which
    // is six observations rather than the fifteen pairwise inversions
    let texts = ["one", "two", "three", "four", "five", "six", "seven"];
    let left = art(texts.iter().map(|text| p([t(*text)])).collect::<Vec<_>>());
    let right = art(texts
        .iter()
        .rev()
        .map(|text| p([t(*text)]))
        .collect::<Vec<_>>());

    let comparison = compare(&left, &right)?;

    assert_eq!(reorders(&comparison).len(), texts.len() - 1);

    Ok(())
}

/// The same pairs are reported as reordered when the two inputs are swapped
#[test]
fn selection_survives_inversion() -> Result<()> {
    for (left, right) in [
        (
            article(["one", "two", "three", "four"]),
            article(["two", "three", "four", "one"]),
        ),
        (
            article(["one", "two", "three", "four"]),
            article(["four", "two", "one", "three"]),
        ),
        (
            article(["one", "two", "three", "four"]),
            article(["three", "one", "four", "two"]),
        ),
    ] {
        let forward = reorders(&compare(&left, &right)?);
        let inverted = reorders(&compare(&right, &left)?.invert());

        assert_eq!(forward, inverted);
    }

    Ok(())
}

/// Repeated runs select the same preserved-order subset
#[test]
fn selection_is_deterministic() -> Result<()> {
    let left = article(["one", "two", "three", "four"]);
    let right = article(["three", "one", "four", "two"]);

    let first = reorders(&compare(&left, &right)?);
    for _ in 0..4 {
        assert_eq!(reorders(&compare(&left, &right)?), first);
    }

    Ok(())
}
