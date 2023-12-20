use codec::{
    common::{
        async_trait::async_trait,
        eyre::Result,
        serde_json::{Map, Value},
    },
    format::Format,
    node_type::NodeType,
    schema::Node,
    status::Status,
    Codec, CodecSupport, DecodeOptions, EncodeOptions, Losses, Mapping,
};

pub mod r#trait;
use r#trait::JsonCodec as _;

/// A codec for JSON
pub struct JsonCodec;

#[async_trait]
impl Codec for JsonCodec {
    fn name(&self) -> &str {
        "json"
    }

    fn status(&self) -> Status {
        Status::Stable
    }

    fn supports_from_format(&self, format: Format) -> CodecSupport {
        match format {
            Format::Json => CodecSupport::NoLoss,
            _ => CodecSupport::None,
        }
    }

    fn supports_from_type(&self, _node_type: NodeType) -> CodecSupport {
        CodecSupport::NoLoss
    }

    async fn from_str(&self, str: &str, _options: Option<DecodeOptions>) -> Result<(Node, Losses)> {
        let node = Node::from_json(str)?;

        Ok((node, Losses::none()))
    }

    fn supports_to_format(&self, format: Format) -> CodecSupport {
        match format {
            Format::Json => CodecSupport::NoLoss,
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
    ) -> Result<(String, Losses, Mapping)> {
        let EncodeOptions {
            standalone,
            compact,
            ..
        } = options.unwrap_or_default();

        if !standalone.unwrap_or_default() {
            return Ok((
                match compact {
                    Some(true) => node.to_json(),
                    Some(false) | None => node.to_json_pretty(),
                }?,
                Losses::none(),
                Mapping::none(),
            ));
        }

        let value = node.to_json_value()?;

        let value = if let (Some(true), Some(r#type)) = (
            standalone,
            value
                .as_object()
                .and_then(|object| object.get("type"))
                .and_then(|r#type| r#type.as_str())
                .map(String::from),
        ) {
            let object = value.as_object().expect("checked above").to_owned();

            // Insert the `$schema` and `@context` at the top of the root
            let mut root = Map::with_capacity(object.len() + 1);
            root.insert(
                String::from("$schema"),
                Value::String(format!("https://stencila.org/{type}.schema.json")),
            );
            root.insert(
                String::from("@context"),
                Value::String(String::from("https://stencila.org/context.jsonld")),
            );
            for (key, value) in object.into_iter() {
                root.insert(key, value);
            }

            Value::Object(root)
        } else {
            value
        };

        Ok((
            match compact {
                Some(true) => value.to_json(),
                Some(false) | None => value.to_json_pretty(),
            }?,
            Losses::none(),
            Mapping::none(),
        ))
    }
}
