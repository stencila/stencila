use std::path::Path;

use codec::{
    common::{async_trait::async_trait, eyre::Result},
    stencila_schema::Node,
    utils::vec_string,
    Codec, CodecTrait, DecodeOptions, EncodeOptions,
};

mod decode;
mod encode;
mod gdoc;

/// A codec for Google Docs
pub struct GdocCodec {}

#[async_trait]
impl CodecTrait for GdocCodec {
    fn spec() -> Codec {
        Codec {
            status: "alpha".to_string(),
            formats: vec_string!["gdoc"],
            root_types: vec_string!["Article"],
            to_string: false,
            to_path: false,
            ..Default::default()
        }
    }

    fn from_str(str: &str, _options: Option<DecodeOptions>) -> Result<Node> {
        decode::decode_sync(str)
    }

    async fn from_str_async(str: &str, _options: Option<DecodeOptions>) -> Result<Node> {
        decode::decode_async(str).await
    }

    async fn to_path(node: &Node, path: &Path, options: Option<EncodeOptions>) -> Result<()> {
        encode::encode(node, path, options).await
    }
}
