///! Encoding-decoding tests
///!
///! These integration tests check that for each format the
///! `encode` and `decode` functions are consistent by doing a
///! round trip conversion of arbitrary instances of nodes.
///!
///! Uses `serde_json::to_value` for assertions because currently unable
///! to compare nodes directly. `serde_json::Value` provides most readable
///! way to compare.
use pretty_assertions::assert_eq;
use proptest::prelude::*;
use stencila::methods::{decode, encode};

mod strategies;
use strategies::{article, node, Freedom};

proptest! {
    // Tests for generic data serialization formats

    // Given the high confidence for encoding/decoding for the
    // following formats the number of test cases is minimal
    #![proptest_config(ProptestConfig::with_cases(5))]

    #[test]
    fn json(input in node(Freedom::Max)) {
        let content = encode::json::encode(&input).unwrap();
        let output = decode::json::decode(&content).unwrap();
        assert_eq!(
            serde_json::to_value(&input).unwrap(),
            serde_json::to_value(&output).unwrap()
        )
    }

    #[test]
    fn yaml(input in node(Freedom::Max)) {
        let content = encode::yaml::encode(&input).unwrap();
        let output = decode::yaml::decode(&content).unwrap();
        assert_eq!(
            serde_json::to_value(&input).unwrap(),
            serde_json::to_value(&output).unwrap()
        )
    }
}

proptest! {
    // Tests for article formats

    // Given the relatively high randomness and complexity of each input
    // we reduce the number of test cases from the default of 256
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn html(input in article(Freedom::Max)) {
        let content = encode::html::encode(&input, None).unwrap();
        let output = decode::html::decode(&content, decode::html::Options::default()).unwrap();
        assert_eq!(
            serde_json::to_value(&input).unwrap(),
            serde_json::to_value(&output).unwrap()
        )
    }

    #[ignore]
    #[test]
    fn md(input in article(Freedom::Min)) {
        let content = encode::md::encode(&input).unwrap();
        let output = decode::md::decode(&content).unwrap();
        assert_eq!(
            serde_json::to_value(&input).unwrap(),
            serde_json::to_value(&output).unwrap()
        )
    }

    #[ignore]
    #[test]
    fn pandoc(input in article(Freedom::Min)) {
        let pandoc = encode::pandoc::encode_node(&input).unwrap();
        let output = decode::pandoc::decode_pandoc(pandoc, &decode::pandoc::Options::default()).unwrap();
        assert_eq!(
            serde_json::to_value(&input).unwrap(),
            serde_json::to_value(&output).unwrap()
        )
    }

}
