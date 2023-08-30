use codec::{
    common::{async_trait::async_trait, eyre::Result},
    format::Format,
    schema::Node,
    status::Status,
    Codec, DecodeOptions, EncodeOptions, Losses,
};

pub mod r#trait;
use r#trait::JsonCodec as _;

#[cfg(test)]
mod tests;

/// A codec for JSON
pub struct JsonCodec;

#[async_trait]
impl Codec for JsonCodec {
    fn name(&self) -> &str {
        "json"
    }

    fn status(&self) -> Status {
        Status::Unstable
    }

    fn supported_formats(&self) -> Vec<Format> {
        vec![Format::Json]
    }

    async fn from_str(&self, str: &str, _options: Option<DecodeOptions>) -> Result<(Node, Losses)> {
        let node = Node::from_json(str)?;

        Ok((node, Losses::none()))
    }

    async fn to_string(
        &self,
        node: &Node,
        options: Option<EncodeOptions>,
    ) -> Result<(String, Losses)> {
        let EncodeOptions { compact, .. } = options.unwrap_or_default();

        let json = match compact {
            true => node.to_json(),
            false => node.to_json_pretty(),
        }?;

        Ok((json, Losses::none()))
    }
}
