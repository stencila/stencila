use codec::{
    common::eyre::Result, stencila_schema::Node, utils::vec_string, Codec, CodecTrait,
    DecodeOptions, EncodeOptions,
};
use codec_md::MdCodec;

#[cfg(feature = "decode")]
mod decode;

#[cfg(feature = "encode")]
mod encode;

/// A codec for R Markdown
pub struct RmdCodec {}

impl CodecTrait for RmdCodec {
    fn spec() -> Codec {
        let md_codec = MdCodec::spec();
        Codec {
            status: "beta".to_string(),
            formats: vec_string!["rmd"],
            root_types: vec_string!["Article"],
            from_string: cfg!(feature = "decode"),
            from_path: cfg!(feature = "decode"),
            to_string: cfg!(feature = "encode"),
            to_path: cfg!(feature = "encode"),
            unsupported_types: md_codec.unsupported_types,
            unsupported_properties: md_codec.unsupported_properties,
        }
    }

    #[cfg(feature = "decode")]
    fn from_str(str: &str, options: Option<DecodeOptions>) -> Result<Node> {
        decode::decode(str, options)
    }

    #[cfg(feature = "encode")]
    fn to_string(node: &Node, options: Option<EncodeOptions>) -> Result<String> {
        encode::encode(node, options)
    }
}
