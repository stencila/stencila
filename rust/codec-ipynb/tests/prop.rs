use codec::CodecTrait;
use codec_ipynb::IpynbCodec;
use test_props::{article, proptest::prelude::*, Freedom};
use test_utils::assert_json_eq;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn test(input in article(
        Freedom::Min,
        IpynbCodec::spec().unsupported_types,
        IpynbCodec::spec().unsupported_properties,
    )) {
        let string = IpynbCodec::to_string(&input, None).unwrap();
        let output = IpynbCodec::from_str(&string, None).unwrap();
        assert_json_eq!(input, output)
    }
}
