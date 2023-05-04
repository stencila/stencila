use codec::{
    common::{
        async_trait::async_trait,
        eyre::Result,
        serde::{de::DeserializeOwned, Serialize},
    },
    format::Format,
    schema::Node,
    status::Status,
    Codec, DecodeOptions, EncodeOptions, Losses,
};

/// A codec for the Rusty Object Notation (RON)
///
/// Mostly useful for Rust developers for for easily inspecting
/// the structure of Stencila documents. See also [`Format::Debug`].
pub struct RonCodec;

#[async_trait]
impl Codec for RonCodec {
    fn name(&self) -> &str {
        "ron"
    }

    fn status(&self) -> Status {
        // Considered unstable because RON does not appear to encode
        // all nodes as expected.
        Status::Unstable
    }

    fn supported_formats(&self) -> Vec<Format> {
        vec![Format::Ron]
    }

    async fn from_str(&self, str: &str, _options: Option<DecodeOptions>) -> Result<(Node, Losses)> {
        let node = Node::from_ron(str)?;

        Ok((node, Losses::new()))
    }

    async fn to_string(
        &self,
        node: &Node,
        options: Option<EncodeOptions>,
    ) -> Result<(String, Losses)> {
        let EncodeOptions { compact, .. } = options.unwrap_or_default();

        let ron = match compact {
            true => node.to_ron(),
            false => node.to_ron_pretty(),
        }?;

        Ok((ron, Losses::new()))
    }
}

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
