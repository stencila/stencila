//! Property-based tests of the invariants, over generated small nodes
//!
//! The targeted fixtures in `invariants.rs` cover the ambiguous cases deliberately;
//! these cover the space around them, so that an algorithm change cannot hold the
//! invariants only on the cases someone thought to write down.
//!
//! Run with one of the `proptest-*` features, which select how large the generated
//! nodes are:
//!
//! ```sh
//! cargo test -p stencila-node-compare --features proptest-min
//! ```

#![allow(unused_imports)]

use std::collections::HashSet;

use proptest::prelude::{ProptestConfig, TestCaseError, prop_assert, prop_assert_eq, proptest};

use stencila_node_compare::{
    Alignment, Side, align, compare, projection::Projection, projections_equal,
};
use stencila_node_path::NodePath;
use stencila_schema::{Article, Node};

/// The number of structured occurrences of a node, plus one for a scalar root
///
/// A scalar root has no occurrence of its own but still receives a root correspondence.
#[cfg(any(
    feature = "proptest-min",
    feature = "proptest-low",
    feature = "proptest-high",
    feature = "proptest-max"
))]
fn expected_records(node: &Node) -> Result<usize, TestCaseError> {
    let projection = Projection::new(node, Side::Left)
        .map_err(|error| TestCaseError::fail(error.to_string()))?;

    Ok(match projection.occurrences().len() {
        0 => 1,
        count => count,
    })
}

/// The left and right paths an alignment records
#[cfg(any(
    feature = "proptest-min",
    feature = "proptest-low",
    feature = "proptest-high",
    feature = "proptest-max"
))]
fn recorded_paths(alignment: &Alignment) -> (Vec<NodePath>, Vec<NodePath>) {
    let mut left = Vec::new();
    let mut right = Vec::new();
    for correspondence in &alignment.correspondences {
        if let Some(node) = correspondence.left() {
            left.push(node.path.clone());
        }
        if let Some(node) = correspondence.right() {
            right.push(node.path.clone());
        }
    }

    (left, right)
}

#[cfg(any(
    feature = "proptest-min",
    feature = "proptest-low",
    feature = "proptest-high",
    feature = "proptest-max"
))]
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Every projected occurrence appears exactly once, and each side's paths are
    /// individually unique
    #[test]
    fn coverage_is_complete_and_single(left: Article, right: Article) {
        let left = Node::Article(left);
        let right = Node::Article(right);

        let alignment = align(&left, &right)
            .map_err(|error| TestCaseError::fail(error.to_string()))?;
        let (left_paths, right_paths) = recorded_paths(&alignment);

        prop_assert_eq!(left_paths.len(), expected_records(&left)?);
        prop_assert_eq!(right_paths.len(), expected_records(&right)?);

        let unique = |paths: &[NodePath]| paths.iter().cloned().collect::<HashSet<_>>().len();
        prop_assert_eq!(unique(&left_paths), left_paths.len());
        prop_assert_eq!(unique(&right_paths), right_paths.len());
    }

    /// Comparing a node with its clone is difference free and path identical
    #[test]
    fn self_comparison_is_difference_free(node: Article) {
        let node = Node::Article(node);
        let clone = node.clone();

        let comparison = compare(&node, &clone)
            .map_err(|error| TestCaseError::fail(error.to_string()))?;

        prop_assert!(comparison.is_equal());
        prop_assert!(comparison.differences.is_empty());
        for (left, right, ..) in comparison.alignment.pairs() {
            prop_assert_eq!(&left.path, &right.path);
        }
    }

    /// Swapping the inputs and inverting the output is the same canonical artifact
    #[test]
    fn swap_and_invert_is_identical(left: Article, right: Article) {
        let left = Node::Article(left);
        let right = Node::Article(right);

        let forward = compare(&left, &right)
            .map_err(|error| TestCaseError::fail(error.to_string()))?;
        let inverted = compare(&right, &left)
            .map_err(|error| TestCaseError::fail(error.to_string()))?
            .invert();

        prop_assert_eq!(forward, inverted);
    }

    /// Repeated runs on identical inputs produce byte-for-byte identical output
    #[test]
    fn runs_are_byte_for_byte_deterministic(left: Article, right: Article) {
        let left = Node::Article(left);
        let right = Node::Article(right);

        let serialize = |node: (&Node, &Node)| -> Result<String, TestCaseError> {
            let comparison = compare(node.0, node.1)
                .map_err(|error| TestCaseError::fail(error.to_string()))?;
            serde_json::to_string(&comparison)
                .map_err(|error| TestCaseError::fail(error.to_string()))
        };

        prop_assert_eq!(serialize((&left, &right))?, serialize((&left, &right))?);
    }

    /// Equality matches canonical projection equality exactly
    #[test]
    fn equality_matches_projection_equality(left: Article, right: Article) {
        let left = Node::Article(left);
        let right = Node::Article(right);

        let comparison = compare(&left, &right)
            .map_err(|error| TestCaseError::fail(error.to_string()))?;
        let equal = projections_equal(&left, &right)
            .map_err(|error| TestCaseError::fail(error.to_string()))?;

        prop_assert_eq!(comparison.is_equal(), equal);
    }

    /// Every reference in a serialized artifact resolves in its original projection
    /// with the node type it records
    #[test]
    fn references_resolve_after_serialization(left: Article, right: Article) {
        let left = Node::Article(left);
        let right = Node::Article(right);

        let alignment = align(&left, &right)
            .map_err(|error| TestCaseError::fail(error.to_string()))?;
        let serialized = serde_json::to_string(&alignment)
            .map_err(|error| TestCaseError::fail(error.to_string()))?;
        let deserialized: Alignment = serde_json::from_str(&serialized)
            .map_err(|error| TestCaseError::fail(error.to_string()))?;

        for (side, node) in [(Side::Left, &left), (Side::Right, &right)] {
            let projection = Projection::new(node, side)
                .map_err(|error| TestCaseError::fail(error.to_string()))?;
            let resolvable: HashSet<_> = projection
                .occurrences()
                .iter()
                .map(|occurrence| (occurrence.path.clone(), occurrence.node_type))
                .collect();

            for correspondence in &deserialized.correspondences {
                let reference = match side {
                    Side::Left => correspondence.left(),
                    Side::Right => correspondence.right(),
                };
                let Some(reference) = reference else { continue };

                prop_assert!(
                    resolvable.contains(&(reference.path.clone(), reference.node_type)),
                    "the {} reference to `{}` does not resolve",
                    side,
                    reference.path
                );
            }
        }
    }
}
