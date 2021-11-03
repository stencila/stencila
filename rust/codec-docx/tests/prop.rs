use codec_docx::DocxCodec;
use codec_trait::Codec;
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
        // TODO: Apply fixes to allow this excluded types to be included
        // Blocks to exclude
        "Heading Table".to_string(),
        // Inlines to exclude
        "AudioObject ImageObject VideoObject Quote".to_string()
    )) {
        RUNTIME.block_on(async {
            let file = tempfile::NamedTempFile::new().unwrap();
            let path = file.path();
            DocxCodec::to_path(&input, &path, None).await.unwrap();
            let output = DocxCodec::from_path(&path, None).await.unwrap();
            assert_json_eq!(input, output)
        })
    }
}
