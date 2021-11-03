use codec::CodecTrait;
use codec_latex::LatexCodec;
use once_cell::sync::Lazy;
use test_props::{article, proptest::prelude::*, Freedom};
use test_utils::assert_json_eq;

static RUNTIME: Lazy<tokio::runtime::Runtime> =
    Lazy::new(|| tokio::runtime::Runtime::new().unwrap());

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn test(input in article(
        Freedom::Min,
        LatexCodec::spec().unsupported_types,
        LatexCodec::spec().unsupported_properties
    )) {
        RUNTIME.block_on(async {
            let latex = LatexCodec::to_string_async(&input, None).await.unwrap();
            let output = LatexCodec::from_str_async(&latex, None).await.unwrap();
            assert_json_eq!(input, output)
        })
    }
}
