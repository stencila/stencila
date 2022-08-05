use codec::CodecTrait;
use codec_json::JsonCodec;
use test_props::{node, proptest::prelude::*, Freedom};
use test_utils::assert_json_eq;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn test(input in node(Freedom::Max, vec![])) {
        let string = JsonCodec::to_string(&input, None).unwrap();
        let output = JsonCodec::from_str(&string, None).unwrap();
        assert_json_eq!(input, output)
    }
}
