//! A codec for R Markdown

use codec_trait::{eyre::Result, stencila_schema::Node, Codec, DecodeOptions, EncodeOptions};

#[cfg(feature = "decode")]
mod decode;

#[cfg(feature = "encode")]
mod encode;

pub struct RmdCodec {}

impl Codec for RmdCodec {
    #[cfg(feature = "decode")]
    fn from_str(str: &str, options: Option<DecodeOptions>) -> Result<Node> {
        decode::decode(str, options)
    }

    #[cfg(feature = "encode")]
    fn to_string(node: &Node, options: Option<EncodeOptions>) -> Result<String> {
        encode::encode(node, options)
    }
}
