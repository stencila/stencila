use codec_pandoc::{decode, encode};
use codec_trait::{
    async_trait::async_trait, eyre::Result, stencila_schema::Node, Codec, DecodeOptions,
    EncodeOptions,
};

/// A codec for LaTeX
pub struct LatexCodec {}

#[async_trait]
impl Codec for LatexCodec {
    async fn from_str_async(str: &str, _options: Option<DecodeOptions>) -> Result<Node> {
        decode(str, None, "latex", &[]).await
    }

    async fn to_string_async(node: &Node, _options: Option<EncodeOptions>) -> Result<String> {
        encode(node, None, "latex", &[]).await
    }
}
