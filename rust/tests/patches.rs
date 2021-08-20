///! Patch generation-application tests
///!
///! These integration tests check that, for various node types,
///! the `diff` and `apply` functions are consistent by doing round
///! trips, both ways, between two instances.
use pretty_assertions::assert_eq;
use proptest::prelude::*;
use stencila::patches::{apply_new, diff};

proptest! {
    /// Any two strings including all unicode graphemes
    #[test]
    fn strings_any(a in any::<String>(), b in any::<String>()) {
        let patch = diff(&a, &b);
        assert_eq!(apply_new(&a, &patch), b)
    }

    /// Zero to ten letters from a restricted range
    ///
    /// This test is useful because `strings_any` has a very low
    /// probability of generating `move` operations (because of the
    /// low probability of the same character appearing twice) and so
    /// was missing a bug associated with that operation.
    ///
    /// Move opertions have since been removed for strings but this
    /// test has been kept anyway.
    #[test]
    fn strings_restricted(a in "[a-e]{0,10}", b in "[a-e]{0,10}") {
        let patch = diff(&a, &b);
        assert_eq!(apply_new(&a, &patch), b)
    }
}
