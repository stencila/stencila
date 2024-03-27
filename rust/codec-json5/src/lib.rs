use codec::{
    common::{async_trait::async_trait, eyre::Result},
    format::Format,
    schema::{Node, NodeType},
    status::Status,
    Codec, CodecSupport, DecodeOptions, EncodeOptions, Losses, Mapping,
};

use codec_json5_trait::Json5Codec as _;

#[cfg(test)]
mod tests;

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

    fn supports_from_format(&self, format: &Format) -> CodecSupport {
        match format {
            Format::Json5 => CodecSupport::NoLoss,
            _ => CodecSupport::None,
        }
    }

    fn supports_from_type(&self, _node_type: NodeType) -> CodecSupport {
        CodecSupport::NoLoss
    }

    async fn from_str(
        &self,
        str: &str,
        _options: Option<DecodeOptions>,
    ) -> Result<(Node, Losses, Mapping)> {
        let node = Node::from_json5(str)?;

        Ok((node, Losses::none(), Mapping::none()))
    }

    fn supports_to_format(&self, format: &Format) -> CodecSupport {
        match format {
            Format::Json5 => CodecSupport::NoLoss,
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
        let EncodeOptions { compact, .. } = options.unwrap_or_default();

        let json5 = match compact {
            Some(true) => node.to_json5(),
            Some(false) | None => node.to_json5_pretty(),
        }?;

        Ok((json5, Losses::none(), Mapping::none()))
    }
}
