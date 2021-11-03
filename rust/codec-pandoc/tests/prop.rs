use codec_pandoc::decode_pandoc;
use codec_pandoc::encode_node;
use test_props::{article, proptest::prelude::*, Freedom};
use test_utils::assert_json_eq;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn test(input in article(
        Freedom::Min,
        // TODO: Apply fixes to allow this excluded types to be included
        // Blocks to exclude
        "Table".to_string(),
        // Inlines to exclude
        "AudioObject ImageObject VideoObject".to_string()
    )) {
        let pandoc = encode_node(&input).unwrap();
        let output = decode_pandoc(pandoc).unwrap();
        assert_json_eq!(input, output);
    }
}
