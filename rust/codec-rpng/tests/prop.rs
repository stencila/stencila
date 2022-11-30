use codec::{
    stencila_schema::{BlockContent, Node},
    CodecTrait,
};
use codec_rpng::RpngCodec;
use test_utils::{assert_json_eq, common::tokio};
use test_utils::{code_chunk, proptest::prelude::*, Freedom};

proptest! {
    // RPNGs can be used for all node types but these tests
    // focus on the types for which they are most predominately used.
    // Given the slowness of generating PNGs only use very few cases.
    #![proptest_config(ProptestConfig::with_cases(3))]

    // Currently using `Low` freedom to ensure that code chunk has some
    // text and do not get "Error -32000: Cannot take screenshot with 0 height." error.
    #[ignore]
    #[test]
    fn test_code_chunk(chunk in code_chunk(Freedom::Low)) {
        let input = if let BlockContent::CodeChunk(chunk) = chunk {
            Node::CodeChunk(chunk)
        } else {
            panic!("Whaaat?!@#!! Expected a `CodeChunk`")
        };

        let content = tokio::runtime::Runtime::new().unwrap().block_on(async {
            RpngCodec::to_string_async(&input, None).await.unwrap()
        });

        let output = RpngCodec::from_str(&content, None).unwrap();
        assert_json_eq!(input, output)
    }
}
