use codec::{
    common::{async_trait::async_trait, eyre::Result},
    format::Format,
    schema::{Node, NodeType},
    status::Status,
    Codec, CodecSupport, EncodeOptions, Losses, Mapping,
};

use codec_text_trait::TextCodec as _;

/// A codec for plain text
pub struct TextCodec;

#[async_trait]
impl Codec for TextCodec {
    fn name(&self) -> &str {
        "text"
    }

    fn status(&self) -> Status {
        Status::Alpha
    }

    fn supports_to_format(&self, format: &Format) -> CodecSupport {
        match format {
            Format::Text => CodecSupport::HighLoss,
            _ => CodecSupport::None,
        }
    }

    fn supports_to_type(&self, node_type: NodeType) -> CodecSupport {
        use CodecSupport::*;
        use NodeType::*;
        match node_type {
            String | Text => NoLoss,
            Null | Boolean | Integer | UnsignedInteger | Number => LowLoss,
            _ => HighLoss,
        }
    }

    async fn to_string(
        &self,
        node: &Node,
        _options: Option<EncodeOptions>,
    ) -> Result<(String, Losses, Mapping)> {
        let (text, losses) = node.to_text();
        Ok((text, losses, Mapping::none()))
    }
}
