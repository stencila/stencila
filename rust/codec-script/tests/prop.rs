use codec::{CodecTrait, DecodeOptions, EncodeOptions};
use codec_script::ScriptCodec;
use test_utils::assert_json_eq;
use test_utils::{article, proptest::prelude::*, Freedom};

proptest! {
    #[test]
    fn test(input in article(
        Freedom::Min,
        ScriptCodec::spec().unsupported_types,
        ScriptCodec::spec().unsupported_properties,
    )) {
        let string = ScriptCodec::to_string(&input, Some(EncodeOptions{format:Some("python".to_string()), ..Default::default()})).unwrap();
        let output = ScriptCodec::from_str(&string, Some(DecodeOptions{format:Some("python".to_string())})).unwrap();
        assert_json_eq!(input, output)
    }
}
