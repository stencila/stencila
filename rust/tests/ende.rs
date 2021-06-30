///! Encoding-decoding tests
///!
///! These integration tests check that for each format the
///! `encode` and `decode` functions are consistent by doing a
///! round trip conversion of arbitrary instances of nodes.
///!
///! Uses `serde_json::to_value` for assertions because currently unable
///! to compare nodes directly. `serde_json::Value` provides most readable
///! way to compare.
use proptest::prelude::*;
use stencila::methods::{decode, encode};

mod strategies;

proptest! {
    // Given the high confidence for encoding/decoding for the
    // following formats the number of test cases is minimal
    #![proptest_config(ProptestConfig::with_cases(5))]

    #[test]
    fn json(input in strategies::article()) {
        let content = encode::json::encode(&input).unwrap();
        let output = decode::json::decode(&content).unwrap();
        assert_eq!(
            serde_json::to_value(&output).unwrap(),
            serde_json::to_value(&input).unwrap()
        )
    }

    #[test]
    fn yaml(input in strategies::article()) {
        let content = encode::yaml::encode(&input).unwrap();
        let output = decode::yaml::decode(&content).unwrap();
        assert_eq!(
            serde_json::to_value(&output).unwrap(),
            serde_json::to_value(&input).unwrap()
        )
    }
}

proptest! {
    // Given the relatively high randomness and complexity of each input
    // we reduce the number of test cases from the default of 256
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn html(input in strategies::article()) {
        let content = encode::html::encode(&input).unwrap();
        let output = decode::html::decode(&content, decode::html::Options::default()).unwrap();
        assert_eq!(
            serde_json::to_value(&output).unwrap(),
            serde_json::to_value(&input).unwrap()
        )
    }

}
