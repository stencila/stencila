use codec::{utils::vec_string, CodecTrait};
use codec_json5::Json5Codec;
use test_utils::assert_json_eq;
use test_utils::{node, proptest::prelude::*, Freedom};

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    // JSON5 does not appear to deal with Unicode characters (?)
    // so this test uses `Low` freedom for now.
    // JSON5 seems to serialize/deserialize `0` and `0.0` for `Number` nodes with
    // a value of zero. So skip `Parameter`s which may have a default, min or max of zero.
    #[test]
    fn test(input in node(Freedom::Low, vec_string!("Parameter"))) {
        let string = Json5Codec::to_string(&input, None).unwrap();
        let output = Json5Codec::from_str(&string, None).unwrap();
        assert_json_eq!(input, output)
    }
}
