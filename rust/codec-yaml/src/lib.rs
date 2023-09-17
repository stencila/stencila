use codec::{
    common::{
        async_trait::async_trait,
        eyre::Result,
        serde_yaml::{self, Mapping, Value},
    },
    format::Format,
    schema::{Node, NodeType},
    status::Status,
    Codec, CodecSupport, DecodeOptions, EncodeOptions, Losses,
};

pub mod r#trait;
use r#trait::YamlCodec as _;

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

    fn supports_from_format(&self, format: Format) -> CodecSupport {
        match format {
            Format::Yaml => CodecSupport::HighLoss,
            _ => CodecSupport::None,
        }
    }

    fn supports_from_type(&self, _node_type: NodeType) -> CodecSupport {
        CodecSupport::NoLoss
    }

    async fn from_str(&self, str: &str, _options: Option<DecodeOptions>) -> Result<(Node, Losses)> {
        let node = Node::from_yaml(str)?;

        Ok((node, Losses::none()))
    }

    fn supports_to_format(&self, format: Format) -> CodecSupport {
        match format {
            Format::Yaml => CodecSupport::HighLoss,
            _ => CodecSupport::None,
        }
    }

    fn supports_to_type(&self, _node_type: NodeType) -> CodecSupport {
        CodecSupport::NoLoss
    }

    async fn to_string(
        &self,
        node: &Node,
        options: Option<EncodeOptions>,
    ) -> Result<(String, Losses)> {
        let EncodeOptions { standalone, .. } = options.unwrap_or_default();

        let value = node.to_yaml_value()?;

        let value = if let (Some(true), Some(r#type)) = (
            standalone,
            value
                .as_mapping()
                .and_then(|mapping| mapping.get("type"))
                .and_then(|r#type| r#type.as_str())
                .map(String::from),
        ) {
            let object = value.as_mapping().expect("checked above").to_owned();

            // Insert the `$schema` and `@context` at the top of the root
            let mut root = Mapping::with_capacity(object.len() + 1);
            root.insert(
                Value::String(String::from("$schema")),
                Value::String(format!("https://stencila.dev/{type}.schema.json")),
            );
            root.insert(
                Value::String(String::from("@context")),
                Value::String(format!("https://stencila.dev/{type}.jsonld")),
            );
            for (key, value) in object.into_iter() {
                root.insert(key, value);
            }

            Value::Mapping(root)
        } else {
            value
        };

        let yaml = serde_yaml::to_string(&value)?;

        Ok((yaml, Losses::none()))
    }
}
