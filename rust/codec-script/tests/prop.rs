use codec::{CodecTrait, DecodeOptions, EncodeOptions};
use codec_script::ScriptCodec;
use test_props::{article, proptest::prelude::*, Freedom};
use test_utils::assert_json_eq;

proptest! {
    #[test]
    fn test(input in article(
        Freedom::Min,
        ScriptCodec::spec().unsupported_types,
        ScriptCodec::spec().unsupported_properties,
    )) {
        let string = ScriptCodec::to_string(&input, Some(EncodeOptions{format:Some("py".to_string()), ..Default::default()})).unwrap();
        let output = ScriptCodec::from_str(&string, Some(DecodeOptions{format:Some("py".to_string())})).unwrap();
        assert_json_eq!(input, output)
    }
}
