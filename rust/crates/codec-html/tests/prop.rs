use codec_trait::Codec;
use codec_html::HtmlCodec;
use test_props::{node, proptest::prelude::*, Freedom};
use test_utils::assert_debug_eq;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn test(input in node(Freedom::Max)) {
        let string = HtmlCodec::to_string(&input, None).unwrap();
        let output = HtmlCodec::from_str(&string).unwrap();
        assert_debug_eq!(input, output)
    }
}
