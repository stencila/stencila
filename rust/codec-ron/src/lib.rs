use codec::{Codec, DecodeOptions, EncodeOptions};
use common::{
    async_trait::async_trait,
    eyre::Result,
    serde::{de::DeserializeOwned, Serialize},
};
use schema::Node;

pub trait FromRon: DeserializeOwned {
    /// Decode a Stencila Schema node from RON
    fn from_ron(ron: &str) -> Result<Self> {
        Ok(ron::from_str(ron)?)
    }
}

impl<T> FromRon for T where T: DeserializeOwned {}

pub trait ToRon: Serialize {
    /// Encode a Stencila Schema node to RON
    fn to_ron(&self) -> Result<String> {
        Ok(ron::to_string(self)?)
    }

    /// Encode a Stencila Schema node to indented RON
    fn to_ron_pretty(&self) -> Result<String> {
        Ok(ron::ser::to_string_pretty(
            self,
            ron::ser::PrettyConfig::default(),
        )?)
    }
}

impl<T> ToRon for T where T: Serialize {}

/// A codec for the Rust Object Notation (RON)
pub struct RonCodec;

#[async_trait]
impl Codec for RonCodec {
    async fn from_str(&self, str: &str, _options: Option<DecodeOptions>) -> Result<Node> {
        Node::from_ron(str)
    }

    async fn to_string(&self, node: &Node, options: Option<EncodeOptions>) -> Result<String> {
        let EncodeOptions { compact, .. } = options.unwrap_or_default();

        match compact {
            true => node.to_ron(),
            false => node.to_ron_pretty(),
        }
    }
}
