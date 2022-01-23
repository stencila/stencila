use codec::{
    eyre::Result, stencila_schema::Node, utils::vec_string, Codec, CodecTrait, DecodeOptions,
    EncodeOptions,
};

#[cfg(feature = "decode")]
mod decode;

#[cfg(feature = "decode")]
pub use decode::decode_fragment;

#[cfg(feature = "encode")]
mod encode;

#[cfg(feature = "encode")]
pub use encode::ToMd;

/// A codec for Markdown
pub struct MdCodec {}

impl CodecTrait for MdCodec {
    fn spec() -> Codec {
        Codec {
            status: "beta".to_string(),
            formats: vec_string!["md"],
            root_types: vec_string!["Article"],
            from_string: cfg!(feature = "decode"),
            from_path: cfg!(feature = "decode"),
            to_string: cfg!(feature = "encode"),
            to_path: cfg!(feature = "encode"),
            ..Default::default()
        }
    }

    #[cfg(feature = "decode")]
    fn from_str(str: &str, _options: Option<DecodeOptions>) -> Result<Node> {
        decode::decode(str)
    }

    #[cfg(feature = "encode")]
    fn to_string(node: &Node, options: Option<EncodeOptions>) -> Result<String> {
        encode::encode(node, options)
    }
}
