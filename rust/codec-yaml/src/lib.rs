use codec::{
    common::{
        async_trait::async_trait,
        eyre::Result,
        serde_yaml::{self, Value},
    },
    format::Format,
    schema::{Article, Node, NodeType},
    status::Status,
    Codec, CodecSupport, DecodeInfo, DecodeOptions, EncodeInfo, EncodeOptions,
};
use version::STENCILA_VERSION;

pub mod r#trait;
use r#trait::YamlCodec as _;

#[cfg(test)]
mod tests;

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

    fn supports_from_format(&self, format: &Format) -> CodecSupport {
        match format {
            Format::Yaml => CodecSupport::NoLoss,
            _ => CodecSupport::None,
        }
    }

    fn supports_to_format(&self, format: &Format) -> CodecSupport {
        match format {
            Format::Yaml => CodecSupport::NoLoss,
            _ => CodecSupport::None,
        }
    }

    fn supports_from_type(&self, _node_type: NodeType) -> CodecSupport {
        CodecSupport::NoLoss
    }

    fn supports_to_type(&self, _node_type: NodeType) -> CodecSupport {
        CodecSupport::NoLoss
    }

    async fn from_str(
        &self,
        str: &str,
        _options: Option<DecodeOptions>,
    ) -> Result<(Node, DecodeInfo)> {
        let mut node = Node::from_yaml(str)?;

        // If the node is a type with a catch-all `extra` property, remove any special
        // keys that may have been added during encoding (see below) which will end up in extras
        if let Node::Article(Article { options, .. }) = &mut node {
            let is_empty = if let Some(extra) = options.extra.as_mut() {
                extra.swap_remove("$schema");
                extra.swap_remove("@context");
                extra.is_empty()
            } else {
                false
            };
            if is_empty {
                options.extra = None;
            }
        }

        Ok((node, DecodeInfo::none()))
    }

    async fn to_string(
        &self,
        node: &Node,
        options: Option<EncodeOptions>,
    ) -> Result<(String, EncodeInfo)> {
        let EncodeOptions { standalone, .. } = options.unwrap_or_default();

        if !standalone.unwrap_or_default() {
            return Ok((node.to_yaml()?, EncodeInfo::none()));
        }

        let value = node.to_yaml_value()?;
        let value = if let Some(r#type) = value
            .as_mapping()
            .and_then(|mapping| mapping.get("type"))
            .and_then(|r#type| r#type.as_str())
            .map(String::from)
        {
            let object = value.as_mapping().expect("checked above").to_owned();

            // Insert the `$schema` and `@context` at the top of the root
            let mut root = serde_yaml::Mapping::with_capacity(object.len() + 1);
            root.insert(
                Value::String(String::from("$schema")),
                Value::String(format!(
                    "https://stencila.org/v{STENCILA_VERSION}/{type}.schema.json"
                )),
            );
            root.insert(
                Value::String(String::from("@context")),
                Value::String(format!(
                    "https://stencila.org/v{STENCILA_VERSION}/context.jsonld"
                )),
            );
            for (key, value) in object.into_iter() {
                root.insert(key, value);
            }

            Value::Mapping(root)
        } else {
            value
        };

        Ok((value.to_yaml()?, EncodeInfo::none()))
    }
}
