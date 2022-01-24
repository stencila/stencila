use codec::{utils::vec_string, CodecTrait};
use codec_docx::DocxCodec;
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
        [
            DocxCodec::spec().unsupported_types,
            // Pandoc replaces the media object with the description if
            // it can not find the file. So exclude these types from the test.
            vec_string!["AudioObject", "ImageObject", "VideoObject"]
        ].concat(),
        DocxCodec::spec().unsupported_properties
    )) {
        RUNTIME.block_on(async {
            let file = tempfile::NamedTempFile::new().unwrap();
            let path = file.path();
            DocxCodec::to_path(&input, path, None).await.unwrap();
            let output = DocxCodec::from_path(path, None).await.unwrap();
            assert_json_eq!(input, output)
        })
    }
}
