///! Patch generation-application tests
///!
///! These integration tests check that, for various node types,
///! the `diff` and `apply` functions are consistent by doing round
///! trips, both ways, between two instances.
use pretty_assertions::assert_eq;
use proptest::prelude::*;
use stencila::patches::{apply_new, diff};

proptest! {
    #[test]
    fn string(a in any::<String>(), b in any::<String>()) {
        let patch = diff(&a, &b);
        assert_eq!(apply_new(&a, &patch), b)
    }
}
