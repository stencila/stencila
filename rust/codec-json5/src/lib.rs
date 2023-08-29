use codec::{
    common::{async_trait::async_trait, eyre::Result},
    format::Format,
    schema::Node,
    status::Status,
    Codec, DecodeOptions, EncodeOptions, Losses,
};

pub mod r#trait;
use r#trait::Json5Codec as _;

/// A codec for JSON5
pub struct Json5Codec;

#[async_trait]
impl Codec for Json5Codec {
    fn name(&self) -> &str {
        "json5"
    }

    fn status(&self) -> Status {
        Status::Stable
    }

    fn supported_formats(&self) -> Vec<Format> {
        vec![Format::Json5]
    }

    async fn from_str(&self, str: &str, _options: Option<DecodeOptions>) -> Result<(Node, Losses)> {
        let node = Node::from_json5(str)?;

        Ok((node, Losses::none()))
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

        Ok((json5, Losses::none()))
    }
}
