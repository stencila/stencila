use codec::CodecTrait;
use codec_rmd::RmdCodec;
use test_utils::assert_json_eq;
use test_utils::{article, proptest::prelude::*, Freedom};

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn test(input in article(
        Freedom::Min,
        RmdCodec::spec().unsupported_types,
        RmdCodec::spec().unsupported_properties,
    ))  {
        let string = RmdCodec::to_string(&input, None).unwrap();
        let output = RmdCodec::from_str(&string, None).unwrap();
        assert_json_eq!(input, output)
    }
}
