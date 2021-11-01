use codec_trait::{eyre::Result, Codec, EncodeOptions};
use stencila_schema::Node;

#[cfg(feature = "decode")]
mod decode;

#[cfg(feature = "encode")]
mod encode;

#[cfg(feature = "encode")]
pub use encode::ToHtml;

pub struct HtmlCodec {}

impl Codec for HtmlCodec {
    #[cfg(feature = "decode")]
    fn from_str(str: &str) -> Result<Node> {
        decode::decode(str, false)
    }

    #[cfg(feature = "encode")]
    fn to_string(node: &Node, options: Option<EncodeOptions>) -> Result<String> {
        encode::encode(node, options)
    }
}
