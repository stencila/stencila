use codec_md::MarkdownCodec;
use codec_trait::Codec;
use test_props::{article, proptest::prelude::*, Freedom};
use test_utils::assert_json_eq;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn test(input in article(
        Freedom::Min,
        // Blocks to exclude
        // TODO: Fix handling of table headers
        "Table".to_string(),
        // Inlines to exclude
        // TODO: Fix these inline nodes that use HTML notation
        "NontextualAnnotation Quote".to_string()
    )) {
        let string = MarkdownCodec::to_string(&input, None).unwrap();
        let output = MarkdownCodec::from_str(&string, None).unwrap();
        assert_json_eq!(input, output)
    }
}
