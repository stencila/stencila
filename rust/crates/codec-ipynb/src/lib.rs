//! A codec for Jupter Notebooks

use codec_trait::{eyre::Result, stencila_schema::Node, Codec, DecodeOptions, EncodeOptions};

#[cfg(feature = "decode")]
mod decode;

#[cfg(feature = "encode")]
mod encode;

#[cfg(feature = "translate")]
mod translate;
pub use translate::*;

pub struct IpynbCodec {}

#[cfg(any(feature = "decode", feature = "encode"))]
impl Codec for IpynbCodec {
    #[cfg(feature = "decode")]
    fn from_str(str: &str, _options: Option<DecodeOptions>) -> Result<Node> {
        decode::decode(str)
    }

    #[cfg(feature = "encode")]
    fn to_string(node: &Node, _options: Option<EncodeOptions>) -> Result<String> {
        encode::encode(node)
    }
}
