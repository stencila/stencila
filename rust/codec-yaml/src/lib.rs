use codec::{
    common::{async_trait::async_trait, eyre::Result},
    format::Format,
    schema::Node,
    status::Status,
    Codec, DecodeOptions, EncodeOptions, Losses,
};

mod from_yaml;
mod to_yaml;

pub use from_yaml::FromYaml;
pub use to_yaml::ToYaml;

/// A codec for YAML
pub struct YamlCodec;

#[async_trait]
impl Codec for YamlCodec {
    fn name(&self) -> &str {
        "yaml"
    }

    fn status(&self) -> Status {
        Status::Stable
    }

    fn supported_formats(&self) -> Vec<Format> {
        vec![Format::Yaml]
    }

    async fn from_str(&self, str: &str, _options: Option<DecodeOptions>) -> Result<(Node, Losses)> {
        let node = Node::from_yaml(str)?;

        Ok((node, Losses::none()))
    }

    async fn to_string(
        &self,
        node: &Node,
        _options: Option<EncodeOptions>,
    ) -> Result<(String, Losses)> {
        let yaml = node.to_yaml()?;

        Ok((yaml, Losses::none()))
    }
}
