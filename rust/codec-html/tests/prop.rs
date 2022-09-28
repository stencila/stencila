use codec::CodecTrait;
use codec_html::HtmlCodec;
use test_props::{article, proptest::prelude::*, Freedom};
use test_utils::assert_json_eq;

proptest! {
    // Given that `Freedom::Max` creates large complex documents,
    // reduce number of test cases to keep runtime below 5s
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn test(input in article(
        Freedom::Max,
        [
            HtmlCodec::spec().unsupported_types,
            vec!["Parameter".to_string(), "For".to_string(), "If".to_string()
        ]].concat(),
        HtmlCodec::spec().unsupported_properties
    )) {
        let string = HtmlCodec::to_string(&input, None).unwrap();
        let output = HtmlCodec::from_str(&string, None).unwrap();
        assert_json_eq!(input, output)
    }
}
