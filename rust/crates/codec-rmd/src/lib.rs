//! A codec for R Markdown

use codec_trait::{eyre::Result, stencila_schema::Node, Codec, EncodeOptions};

#[cfg(feature = "decode")]
mod decode;

#[cfg(feature = "encode")]
mod encode;

pub struct RmdCodec {}

impl Codec for RmdCodec {
    #[cfg(feature = "decode")]
    fn from_str(str: &str) -> Result<Node> {
        decode::decode(str)
    }

    #[cfg(feature = "encode")]
    fn to_string(node: &Node, _options: Option<EncodeOptions>) -> Result<String> {
        encode::encode(node)
    }
}
