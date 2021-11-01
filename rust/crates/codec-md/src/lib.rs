//! A codec for Markdown

use codec_trait::{eyre::Result, stencila_schema::Node, Codec, EncodeOptions};

#[cfg(feature = "decode")]
mod decode;

#[cfg(feature = "decode")]
pub use decode::decode_fragment;

#[cfg(feature = "encode")]
mod encode;

#[cfg(feature = "encode")]
pub use encode::ToMd;

pub struct MarkdownCodec {}

impl Codec for MarkdownCodec {
    #[cfg(feature = "decode")]
    fn from_str(str: &str) -> Result<Node> {
        decode::decode(str)
    }

    #[cfg(feature = "encode")]
    fn to_string(node: &Node, options: Option<EncodeOptions>) -> Result<String> {
        encode::encode(node, options)
    }
}
