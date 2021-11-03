use codec::CodecTrait;
use codec_md::MarkdownCodec;
use test_props::{article, proptest::prelude::*, Freedom};
use test_utils::assert_json_eq;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn test(input in article(
        Freedom::Min,
        MarkdownCodec::spec().unsupported_types,
        MarkdownCodec::spec().unsupported_properties,
    )) {
        let string = MarkdownCodec::to_string(&input, None).unwrap();
        let output = MarkdownCodec::from_str(&string, None).unwrap();
        assert_json_eq!(input, output)
    }
}
