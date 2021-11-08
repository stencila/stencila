use codec::CodecTrait;
use codec_md::MdCodec;
use test_props::{article, proptest::prelude::*, Freedom};
use test_utils::assert_json_eq;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn test(input in article(
        Freedom::Min,
        MdCodec::spec().unsupported_types,
        MdCodec::spec().unsupported_properties,
    )) {
        let string = MdCodec::to_string(&input, None).unwrap();
        let output = MdCodec::from_str(&string, None).unwrap();
        assert_json_eq!(input, output)
    }
}
