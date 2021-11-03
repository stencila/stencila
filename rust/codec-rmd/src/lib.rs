use codec::{
    eyre::Result, stencila_schema::Node, utils::vec_string, Codec, CodecTrait, DecodeOptions,
    EncodeOptions,
};

#[cfg(feature = "decode")]
mod decode;

#[cfg(feature = "encode")]
mod encode;

/// A codec for markdown
pub struct RmdCodec {}

impl CodecTrait for RmdCodec {
    fn spec() -> Codec {
        Codec {
            status: "alpha".to_string(),
            formats: vec_string!["rmd"],
            root_types: vec_string!["Article"],
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
    fn from_str(str: &str, options: Option<DecodeOptions>) -> Result<Node> {
        decode::decode(str, options)
    }

    #[cfg(feature = "encode")]
    fn to_string(node: &Node, options: Option<EncodeOptions>) -> Result<String> {
        encode::encode(node, options)
    }
}
