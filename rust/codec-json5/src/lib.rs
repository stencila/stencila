use codec::{
    common::{
        async_trait::async_trait,
        eyre::Result,
        json5,
        serde::{de::DeserializeOwned, Serialize},
        serde_json,
    },
    format::Format,
    schema::Node,
    status::Status,
    Codec, DecodeOptions, EncodeOptions, Losses,
};

/// A codec for JSON5
pub struct Json5Codec;

#[async_trait]
impl Codec for Json5Codec {
    fn name(&self) -> &str {
        "json5"
    }

    fn status(&self) -> Status {
        // Considered unstable until `to_string` encodes JSON5, not JSON
        Status::Unstable
    }

    fn supported_formats(&self) -> Vec<Format> {
        vec![Format::Json5]
    }

    async fn from_str(&self, str: &str, _options: Option<DecodeOptions>) -> Result<(Node, Losses)> {
        let node = Node::from_json5(str)?;

        Ok((node, Losses::new()))
    }

    async fn to_string(
        &self,
        node: &Node,
        options: Option<EncodeOptions>,
    ) -> Result<(String, Losses)> {
        let EncodeOptions { compact, .. } = options.unwrap_or_default();

        let json5 = match compact {
            true => node.to_json5(),
            false => node.to_json5_pretty(),
        }?;

        Ok((json5, Losses::new()))
    }
}

pub trait FromJson5: DeserializeOwned {
    /// Decode a Stencila Schema node from JSON5
    fn from_json5(json5: &str) -> Result<Self> {
        Ok(json5::from_str(json5)?)
    }
}

impl<T> FromJson5 for T where T: DeserializeOwned {}

pub trait ToJson5: Serialize {
    /// Encode a Stencila Schema node to JSON5
    ///
    /// Note: at the time of writing, the `json5` actually produces
    /// JSON output (which is of course valid JSON5, but less concise).
    fn to_json5(&self) -> Result<String>
    where
        Self: Sized,
    {
        Ok(json5::to_string(self)?)
    }

    /// Encode a Stencila Schema node to indented JSON5
    fn to_json5_pretty(&self) -> Result<String>
    where
        Self: Sized,
    {
        // Use `serde_json` here for indentation (which `json5` crate lacks)
        Ok(serde_json::to_string_pretty(self)?)
    }
}

impl<T> ToJson5 for T where T: Serialize {}
