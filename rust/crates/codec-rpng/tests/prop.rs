use codec_rpng::RpngCodec;
use codec_trait::{
    stencila_schema::{BlockContent, Node},
    Codec,
};
use test_props::{code_chunk, proptest::prelude::*, Freedom};
use test_utils::assert_debug_eq;

proptest! {
    // RPNGs can be used for all node types but these tests
    // focus on the types for which they are most predominately used.
    // Given the slowness of generating PNGs only use very few cases.
    #![proptest_config(ProptestConfig::with_cases(3))]

    #[test]
    fn test_code_chunk(chunk in code_chunk(Freedom::Max)) {
        let input = if let BlockContent::CodeChunk(chunk) = chunk {
            Node::CodeChunk(chunk)
        } else {
            panic!("Whaaat?!@#!! Expected a `CodeChunk`")
        };

        let content = tokio::runtime::Runtime::new().unwrap().block_on(async {
            RpngCodec::to_string_async(&input, None).await.unwrap()
        });

        let output = RpngCodec::from_str(&content).unwrap();
        assert_debug_eq!(input, output)
    }
}
