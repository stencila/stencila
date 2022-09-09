///! Patch generation-application tests
///!
///! These integration tests check that, for various node types,
///! the `diff` and `apply` functions are consistent by doing round
///! trips, both ways, between two instances.
use node_patch::{apply_new, diff};
use test_props::{
    block_content, inline_content,
    proptest::{
        collection::{btree_map, size_range, vec},
        prelude::*,
    },
    vec_block_content, vec_inline_content, Freedom,
};
use test_utils::assert_json_eq;

proptest! {
    // Higher number of cases than the default because some patches can fail
    // on rare corner cases (in particular `Move` operations).
    #![proptest_config(ProptestConfig::with_cases(1000))]

    /// Any two strings including all unicode graphemes
    #[test]
    fn strings_any(
        a in any::<String>(),
        b in any::<String>()
    ) {
        let patch = diff(&a, &b);
        assert_eq!(apply_new(&a, &patch).unwrap(), b)
    }

    /// Zero to ten letters from a restricted range
    ///
    /// This test is useful because `strings_any` has a very low
    /// probability of generating `Move` operations (because of the
    /// low probability of the same character appearing twice) and so
    /// was missing a bug associated with that operation. Move operations
    /// have since been removed for strings but this test has been kept anyway.
    #[test]
    fn strings_restricted(
        a in "[a-e]{0,10}",
        b in "[a-e]{0,10}"
    ) {
        let patch = diff(&a, &b);
        assert_eq!(apply_new(&a, &patch).unwrap(), b)
    }

    // Vectors of integers
    #[test]
    fn vecs_integers(
        a in vec(0..10i64, size_range(0..10)),
        b in vec(0..10i64, size_range(0..10))
    ) {
        let patch = diff(&a, &b);
        assert_eq!(apply_new(&a, &patch).unwrap(), b)
    }

    // Vectors of strings (which are themselves `Patchable` so
    // may have internal `Add`, `Remove`, `Replace` operations)
    #[test]
    fn vecs_strings(
        a in vec("[a-e]{0,5}", size_range(0..10)),
        b in vec("[a-e]{0,5}", size_range(0..10))
    ) {
        let patch = diff(&a, &b);
        assert_eq!(apply_new(&a, &patch).unwrap(), b)
    }

    // Map of integers
    #[test]
    fn btree_map_integers(
        a in btree_map("[a-e]{0,2}", 0..10i64, size_range(0..100)),
        b in btree_map("[a-e]{0,2}", 0..10i64, size_range(0..100))
    ) {
        let patch = diff(&a, &b);
        assert_eq!(apply_new(&a, &patch).unwrap(), b)
    }

    // Map of strings (which are themselves `Patchable` so
    // may have internal `Add`, `Remove`, `Replace` operations)
    #[test]
    fn btree_map_strings(
        a in btree_map("[a-e]{0,2}", "[a-e]{0,5}", size_range(0..100)),
        b in btree_map("[a-e]{0,2}", "[a-e]{0,5}", size_range(0..100))
    ) {
        let patch = diff(&a, &b);
        assert_eq!(apply_new(&a, &patch).unwrap(), b)
    }
}

proptest! {
    // Reduced number of cases for these more complicated tests.
    #![proptest_config(ProptestConfig::with_cases(100))]

    // Inlines
    #[test]
    fn inlines(
        a in inline_content(Freedom::Low, vec![]),
        b in inline_content(Freedom::Low, vec![])
    ) {
        let patch = diff(&a, &b);
        assert_json_eq!(apply_new(&a, &patch).unwrap(), b)
    }

    // Blocks
    #[test]
    fn blocks(
        a in block_content(Freedom::Low, vec![]),
        b in block_content(Freedom::Low, vec![])
    ) {
        let patch = diff(&a, &b);
        assert_json_eq!(apply_new(&a, &patch).unwrap(), b)
    }

    // Vectors of inline content
    #[test]
    fn vecs_inlines(
        a in vec_inline_content(Freedom::Low, vec![]),
        b in vec_inline_content(Freedom::Low, vec![]),
    ) {
        let patch = diff(&a, &b);
        assert_json_eq!(apply_new(&a, &patch).unwrap(), b)
    }

    // Vectors of block content
    #[test]
    fn vecs_blocks(
        // TODO: For some reason, related to the fact that EnumValidator uses `replaceable_struct!` macro,
        // this test fails. Excluded for now but in the long term, fix, or do not use `replaceable_struct!`.
        a in vec_block_content(Freedom::Low, vec!["EnumValidator".to_string()]),
        b in vec_block_content(Freedom::Low, vec!["EnumValidator".to_string()]),
    ) {
        let patch = diff(&a, &b);
        assert_json_eq!(apply_new(&a, &patch).unwrap(), b)
    }
}
