use codec_ipynb::IpynbCodec;
use codec_trait::Codec;
use test_props::{node, proptest::prelude::*, Freedom};
use test_utils::assert_debug_eq;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn test(input in node(Freedom::Min)) {
        let string = IpynbCodec::to_string(&input, None).unwrap();
        let output = IpynbCodec::from_str(&string, None).unwrap();
        assert_debug_eq!(input, output)
    }
}
