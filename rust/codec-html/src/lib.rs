use codec::{eyre::Result, utils::vec_string, Codec, CodecTrait, DecodeOptions, EncodeOptions};
use stencila_schema::Node;

#[cfg(feature = "decode")]
mod decode;

#[cfg(feature = "decode")]
pub use decode::decode_fragment;

#[cfg(feature = "encode")]
mod encode;

#[cfg(feature = "encode")]
pub use encode::{wrap_standalone, EncodeContext, ToHtml};

/// A codec for HTML
pub struct HtmlCodec {}

impl CodecTrait for HtmlCodec {
    fn spec() -> Codec {
        Codec {
            formats: vec_string!["html"],
            root_types: vec_string!["*"],
            from_string: cfg!(feature = "decode"),
            from_path: cfg!(feature = "decode"),
            to_string: cfg!(feature = "encode"),
            to_path: cfg!(feature = "encode"),
            ..Default::default()
        }
    }

    #[cfg(feature = "decode")]
    fn from_str(str: &str, _options: Option<DecodeOptions>) -> Result<Node> {
        decode::decode(str, false)
    }

    #[cfg(feature = "encode")]
    fn to_string(node: &Node, options: Option<EncodeOptions>) -> Result<String> {
        encode::encode(node, options)
    }
}
