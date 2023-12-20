//! Property-based tests of roundtrip dump/load of nodes to/from Automerge store

#![allow(unused_imports)]

use common::eyre::Result;
use common_dev::{
    pretty_assertions::assert_eq,
    proptest::prelude::{proptest, ProptestConfig},
};
use node_store::{ReadNode, WriteCrdt, WriteNode};
use schema::Article;

#[cfg(any(
    feature = "proptest-min",
    feature = "proptest-low",
    feature = "proptest-high",
    feature = "proptest-max"
))]
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn article(node: Article) {
        let mut store = WriteCrdt::default();
        node.dump(&mut store).unwrap();
        let roundtrip = Article::load(&store).unwrap();

        assert_eq!(roundtrip, node);
    }
}
