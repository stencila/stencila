use codec::{
    eyre::Result, stencila_schema::Node, utils::vec_string, Codec, CodecTrait, DecodeOptions,
    EncodeOptions,
};

#[cfg(feature = "decode")]
mod decode;

#[cfg(feature = "encode")]
mod encode;

#[cfg(feature = "translate")]
mod translate;
pub use translate::*;

// A codec for Jupter Notebook (.ipynb) files
pub struct IpynbCodec {}

#[cfg(any(feature = "decode", feature = "encode"))]
impl CodecTrait for IpynbCodec {
    fn spec() -> Codec {
        Codec {
            status: "alpha".to_string(),
            formats: vec_string!["html"],
            root_types: vec_string!["*"],
            from_string: cfg!(feature = "decode"),
            from_path: cfg!(feature = "decode"),
            to_string: cfg!(feature = "encode"),
            to_path: cfg!(feature = "encode"),
            unsupported_types: vec_string![
                // TODO: Remove these when they are fixed in Markdown codec
                "Table",
                "NontextualAnnotation",
                "Quote"
            ],
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
