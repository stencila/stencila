use codec_json::JsonCodec;
use codec_trait::Codec;
use test_props::{node, proptest::prelude::*, Freedom};
use test_utils::assert_debug_eq;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn test(input in node(Freedom::Max)) {
        let string = JsonCodec::to_string(&input).unwrap();
        let output = JsonCodec::from_str(&string).unwrap();
        assert_debug_eq(&input, &output)
    }
}
