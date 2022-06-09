use codec::{
    common::{async_trait::async_trait, eyre::Result},
    stencila_schema::Node,
    utils::vec_string,
    Codec, CodecTrait, DecodeOptions, EncodeOptions,
};
use codec_pandoc::{decode, encode, PandocCodec};

/// A codec for LaTeX
pub struct LatexCodec {}

#[async_trait]
impl CodecTrait for LatexCodec {
    fn spec() -> Codec {
        let pandoc_codec = PandocCodec::spec();
        Codec {
            status: "alpha".to_string(),
            formats: vec_string!["latex"],
            root_types: vec_string!["Article"],
            unsupported_types: [
                pandoc_codec.unsupported_types,
                vec_string![
                    // TODO: Add support for these. See https://github.com/stencila/encoda/blob/master/src/codecs/latex/__fixtures__/code.tex
                    "CodeChunk",
                    "CodeExpression"
                ],
            ]
            .concat(),
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
