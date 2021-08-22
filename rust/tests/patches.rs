///! Patch generation-application tests
///!
///! These integration tests check that, for various node types,
///! the `diff` and `apply` functions are consistent by doing round
///! trips, both ways, between two instances.
use pretty_assertions::assert_eq;
use proptest::collection::{size_range, vec};
use proptest::prelude::*;
use stencila::patches::{apply_new, diff};

mod strategies;
use strategies::{inline_content, Freedom};

macro_rules! assert_json_eq {
    ($expr1:expr, $expr2:expr) => {
        pretty_assertions::assert_eq!(
            serde_json::to_value(&$expr1).unwrap(),
            serde_json::to_value(&$expr2).unwrap()
        );
    };
}

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
        assert_eq!(apply_new(&a, &patch), b)
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
        assert_eq!(apply_new(&a, &patch), b)
    }

    // Any two inlines
    #[test]
    fn inlines(
        a in inline_content(Freedom::Low),
        b in inline_content(Freedom::Low)
    ) {
        let patch = diff(&a, &b);
        assert_json_eq!(apply_new(&a, &patch), b)
    }

    // Any two vectors of integers
    #[test]
    fn vec_integers(
        a in vec(0..10i64, size_range(0..10)),
        b in vec(0..10i64, size_range(0..10))
    ) {
        let patch = diff(&a, &b);
        assert_json_eq!(apply_new(&a, &patch), b)
    }
}
