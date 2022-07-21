use codec::{
    common::eyre::Result, stencila_schema::Node, utils::vec_string, Codec, CodecTrait,
    DecodeOptions, EncodeOptions,
};

#[cfg(feature = "decode")]
mod decode;

#[cfg(feature = "encode")]
mod encode;

#[cfg(feature = "translate")]
mod translate;
use codec_md::MdCodec;
pub use translate::*;

// A codec for Jupyter Notebook (.ipynb) files
pub struct IpynbCodec {}

#[cfg(any(feature = "decode", feature = "encode"))]
impl CodecTrait for IpynbCodec {
    fn spec() -> Codec {
        let md_codec = MdCodec::spec();
        Codec {
            status: "beta".to_string(),
            formats: vec_string!["ipynb"],
            root_types: vec_string!["Article"],
            from_string: cfg!(feature = "decode"),
            from_path: cfg!(feature = "decode"),
            to_string: cfg!(feature = "encode"),
            to_path: cfg!(feature = "encode"),
            unsupported_types: md_codec.unsupported_types,
            unsupported_properties: md_codec.unsupported_properties,
            ..Default::default()
        }
    }

    #[cfg(feature = "decode")]
    fn from_str(str: &str, _options: Option<DecodeOptions>) -> Result<Node> {
        decode::decode(str)
    }

    #[cfg(feature = "encode")]
    fn to_string(node: &Node, _options: Option<EncodeOptions>) -> Result<String> {
        encode::encode(node)
    }
}
