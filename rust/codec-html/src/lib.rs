use codec::{
    common::{async_trait::async_trait, eyre::Result},
    format::Format,
    schema::{Node, NodeType},
    status::Status,
    Codec, CodecSupport, DecodeInfo, DecodeOptions, EncodeInfo, EncodeOptions,
};

mod decode;
mod encode;

/// A codec for HTML
pub struct HtmlCodec;

#[async_trait]
impl Codec for HtmlCodec {
    fn name(&self) -> &str {
        "html"
    }

    fn status(&self) -> Status {
        Status::UnderDevelopment
    }

    fn supports_from_format(&self, format: &Format) -> CodecSupport {
        match format {
            Format::Html => CodecSupport::LowLoss,
            _ => CodecSupport::None,
        }
    }

    fn supports_to_format(&self, format: &Format) -> CodecSupport {
        match format {
            Format::Html => CodecSupport::LowLoss,
            _ => CodecSupport::None,
        }
    }

    fn supports_to_type(&self, node_type: NodeType) -> CodecSupport {
        use CodecSupport::*;
        use NodeType::*;
        match node_type {
            // Data
            String | Cord => NoLoss,
            // Prose Inlines
            Text | Emphasis | Strong | Subscript | Superscript | Underline => NoLoss,
            // Prose Blocks
            Section | Heading | Paragraph | ThematicBreak => NoLoss,
            // Code
            CodeInline | CodeBlock => NoLoss,
            // Fallback to low loss
            _ => LowLoss,
        }
    }

    async fn from_str(
        &self,
        str: &str,
        options: Option<DecodeOptions>,
    ) -> Result<(Node, DecodeInfo)> {
        decode::decode(str, options)
    }

    async fn to_string(
        &self,
        node: &Node,
        options: Option<EncodeOptions>,
    ) -> Result<(String, EncodeInfo)> {
        encode::encode(node, options)
    }
}
