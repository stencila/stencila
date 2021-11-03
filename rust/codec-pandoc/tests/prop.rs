use codec_pandoc::decode::decode_pandoc;
use codec_pandoc::encode::encode_node;
use test_props::{article, proptest::prelude::*, Freedom};
use test_utils::assert_debug_eq;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn test(input in article(Freedom::Min)) {
        let pandoc = encode_node(&input).unwrap();
        let output = decode_pandoc(pandoc).unwrap();
        assert_debug_eq!(input, output);
    }
}
