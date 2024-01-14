use codec::{
    common::{async_trait::async_trait, eyre::Result},
    format::Format,
    schema::{Node, NodeType},
    status::Status,
    Codec, CodecSupport, DecodeOptions, EncodeOptions, Losses,
};

use codec_markdown_trait::{MarkdownCodec as _, MarkdownEncodeContext};

mod decode;

/// A codec for Markdown
pub struct MarkdownCodec;

#[async_trait]
impl Codec for MarkdownCodec {
    fn name(&self) -> &str {
        "markdown"
    }

    fn status(&self) -> Status {
        Status::Alpha
    }

    fn supports_from_format(&self, format: Format) -> CodecSupport {
        use CodecSupport::*;
        match format {
            Format::Markdown => LowLoss,
            _ => None,
        }
    }

    fn supports_to_format(&self, format: Format) -> CodecSupport {
        use CodecSupport::*;
        match format {
            Format::Markdown => LowLoss,
            _ => None,
        }
    }

    fn supports_from_type(&self, node_type: NodeType) -> CodecSupport {
        use CodecSupport::*;
        use NodeType::*;
        match node_type {
            // Data
            String | Cord => NoLoss,
            Null | Boolean | Integer | UnsignedInteger | Number => LowLoss,
            // Prose Inlines
            Text | Emphasis | Strong | Subscript | Superscript | Underline => NoLoss,
            Link | Parameter | AudioObject | ImageObject | MediaObject | Note => LowLoss,
            // Prose Blocks
            Admonition | StyledBlock | Section | Heading | Paragraph | QuoteBlock
            | ThematicBreak => NoLoss,
            List | ListItem | Table | TableRow | TableCell => LowLoss,
            // Code
            CodeInline | CodeBlock => NoLoss,
            CodeExpression | CodeChunk => LowLoss,
            // Math
            MathInline | MathBlock => NoLoss,
            // Works,
            Article => LowLoss,
            _ => None,
        }
    }

    fn supports_to_type(&self, node_type: NodeType) -> CodecSupport {
        use CodecSupport::*;
        use NodeType::*;
        match node_type {
            // Data
            String | Cord => NoLoss,
            Null | Boolean | Integer | UnsignedInteger | Number => LowLoss,
            // Prose Inlines
            Text | Emphasis | Strong | Subscript | Superscript | Underline => NoLoss,
            Link | Parameter | AudioObject | ImageObject | MediaObject | Note => LowLoss,
            // Prose Blocks
            Admonition | StyledBlock | Section | Heading | Paragraph | QuoteBlock
            | ThematicBreak => NoLoss,
            List | ListItem | Table | TableRow | TableCell => LowLoss,
            // Code
            CodeInline | CodeBlock => NoLoss,
            CodeExpression | CodeChunk => LowLoss,
            // Math
            MathInline | MathBlock => NoLoss,
            // Works,
            Article => LowLoss,
            // Because `to_markdown` is implemented for all types, defaulting to
            // `to_text`, fallback to high loss
            _ => HighLoss,
        }
    }

    async fn from_str(&self, str: &str, options: Option<DecodeOptions>) -> Result<(Node, Losses)> {
        decode::decode(str, options)
    }

    async fn to_string(
        &self,
        node: &Node,
        _options: Option<EncodeOptions>,
    ) -> Result<(String, Losses)> {
        let mut context = MarkdownEncodeContext::default();
        node.to_markdown(&mut context);

        let markdown = context.content.trim().to_string();

        Ok((markdown, context.losses))
    }
}
