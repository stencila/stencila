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
            vec![
                // Exclude types for which decoding support is not yet enabled
                "Parameter".to_string(),
                "Button".to_string(),
                "Form".to_string(),
                "For".to_string(),
                "If".to_string(),
                "Division".to_string(),
                "Span".to_string()
            ]
        ].concat(),
        HtmlCodec::spec().unsupported_properties
    )) {
        let string = HtmlCodec::to_string(&input, None).unwrap();
        let output = HtmlCodec::from_str(&string, None).unwrap();
        assert_json_eq!(input, output)
    }
}
