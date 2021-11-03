use codec::{
    async_trait::async_trait, eyre::Result, stencila_schema::Node, utils::vec_string, Codec,
    CodecTrait, DecodeOptions, EncodeOptions,
};
use codec_pandoc::{decode, encode};

/// A codec for LaTeX
pub struct LatexCodec {}

#[async_trait]
impl CodecTrait for LatexCodec {
    fn spec() -> Codec {
        Codec {
            status: "alpha".to_string(),
            formats: vec_string!["latex"],
            root_types: vec_string!["Article"],
            from_string: true,
            from_path: true,
            to_string: true,
            to_path: true,
            unsupported_types: vec_string![
                // TODO: Fix these
                "Heading",
                "Table",
                "CodeChunk",
                "CodeExpression",
                "AudioObject",
                "ImageObject",
                "VideoObject"
            ],
            ..Default::default()
        }
    }

    async fn from_str_async(str: &str, _options: Option<DecodeOptions>) -> Result<Node> {
        decode(str, None, "latex", &[]).await
    }

    async fn to_string_async(node: &Node, _options: Option<EncodeOptions>) -> Result<String> {
        encode(node, None, "latex", &[]).await
    }
}
