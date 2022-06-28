use codec::CodecTrait;
use codec_pandoc::{decode_pandoc, encode_node, PandocCodec};
use test_props::{article, proptest::prelude::*, Freedom};
use test_utils::assert_json_eq;

proptest! {
    #[test]
    fn test(input in article(
        Freedom::Min,
        PandocCodec::spec().unsupported_types,
        PandocCodec::spec().unsupported_properties,
    )) {
        let pandoc = encode_node(&input, None).unwrap();
        let output = decode_pandoc(pandoc).unwrap();
        assert_json_eq!(input, output);
    }
}
