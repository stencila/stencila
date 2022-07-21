use std::{collections::BTreeMap, path::Path};

use codec::{
    common::{
        async_trait::async_trait,
        eyre::{bail, Result},
    },
    stencila_schema::Node,
    utils::vec_string,
    Codec, CodecTrait, DecodeOptions, EncodeOptions,
};
use node_address::Address;

type NodeRanges = BTreeMap<Address, (i64, i64)>;

mod decode;
mod encode;
mod gdoc;
mod remote;

/// A codec for Google Docs
pub struct GdocCodec {}

#[async_trait]
impl CodecTrait for GdocCodec {
    fn spec() -> Codec {
        Codec {
            status: "beta".to_string(),
            formats: vec_string!["gdoc"],
            root_types: vec_string!["Article"],
            to_string: false,
            has_remote: true,
            ..Default::default()
        }
    }

    fn from_str(str: &str, _options: Option<DecodeOptions>) -> Result<Node> {
        let (node, ..) = decode::decode_sync(str)?;
        Ok(node)
    }

    async fn from_str_async(str: &str, _options: Option<DecodeOptions>) -> Result<Node> {
        let (node, ..) = decode::decode_async(str).await?;
        Ok(node)
    }

    async fn from_remote(path: &Path, _options: Option<DecodeOptions>) -> Result<Node> {
        let (node, ..) = remote::pull(path).await?;
        Ok(node)
    }

    async fn to_path(node: &Node, path: &Path, options: Option<EncodeOptions>) -> Result<()> {
        encode::encode(node, path, options).await
    }

    async fn to_remote(_node: &Node, _path: &Path, _options: Option<EncodeOptions>) -> Result<()> {
        bail!("Unable to push changes to an existing Google Doc")
    }
}
