use codec::CodecTrait;
use codec_rmd::RmdCodec;
use test_props::{article, proptest::prelude::*, Freedom};
use test_utils::assert_json_eq;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn test(input in article(
        Freedom::Min,
        [
            RmdCodec::spec().unsupported_types,
            // Markdown parser does not decode double tilde's without
            // surrounding spaces, so exclude from these tests.
            vec!["Strikeout".to_string()]
        ].concat(),
        RmdCodec::spec().unsupported_properties,
    ))  {
        let string = RmdCodec::to_string(&input, None).unwrap();
        let output = RmdCodec::from_str(&string, None).unwrap();
        assert_json_eq!(input, output)
    }
}
