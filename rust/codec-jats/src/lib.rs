use codec::{
    common::{async_trait::async_trait, eyre::Result},
    format::Format,
    schema::{Node, NodeType},
    status::Status,
    Codec, CodecSupport, DecodeOptions, EncodeOptions, Losses, Mapping,
};

mod decode;
mod encode;

#[cfg(test)]
mod tests;

/// A codec for JATS
pub struct JatsCodec;

#[async_trait]
impl Codec for JatsCodec {
    fn name(&self) -> &str {
        "jats"
    }

    fn status(&self) -> Status {
        Status::UnderDevelopment
    }

    fn supports_from_format(&self, format: &Format) -> CodecSupport {
        use CodecSupport::*;
        match format {
            Format::Jats => LowLoss,
            _ => None,
        }
    }

    fn supports_to_format(&self, format: &Format) -> CodecSupport {
        use CodecSupport::*;
        match format {
            Format::Jats => LowLoss,
            _ => None,
        }
    }

    fn supports_from_type(&self, node_type: NodeType) -> CodecSupport {
        use CodecSupport::*;
        use NodeType::*;
        match node_type {
            // Prose Inlines
            Text | Emphasis | Strong | Strikeout | Subscript | Superscript | Underline
            | QuoteInline | StyledInline | Note => NoLoss,
            Link | AudioObject | ImageObject | VideoObject => LowLoss,
            // Prose Blocks
            Admonition | Section | Heading | Paragraph | QuoteBlock | ThematicBreak => NoLoss,
            // Math
            MathInline | MathBlock => LowLoss,
            // Code
            CodeInline => NoLoss,
            CodeExpression => LowLoss,
            // Data
            String | Cord | Date | DateTime | Time | Timestamp | Duration => NoLoss,
            // Works
            Article => LowLoss,
            _ => None,
        }
    }

    fn supports_to_type(&self, node_type: NodeType) -> CodecSupport {
        use CodecSupport::*;
        use NodeType::*;
        match node_type {
            // Prose Inlines
            Text | Emphasis | Strong | Strikeout | Subscript | Superscript | Underline
            | InsertInline | QuoteInline | StyledInline | Note => NoLoss,
            Link | AudioObject | ImageObject | VideoObject => LowLoss,
            DeleteInline => HighLoss,
            // Prose Blocks
            Admonition | Section | Heading | Paragraph | QuoteBlock | ThematicBreak => NoLoss,
            List | ListItem | Figure => LowLoss,
            // Math
            MathInline | MathBlock => NoLoss,
            // Code
            CodeInline | CodeBlock => NoLoss,
            CodeExpression | CodeChunk => LowLoss,
            // Data
            String | Cord | Date | DateTime | Time | Timestamp | Duration => NoLoss,
            Null | Boolean | Integer | UnsignedInteger | Number => LowLoss,
            // Works
            Article | Claim => LowLoss,
            // Other
            Organization | PostalAddress | Product => LowLoss,
            // If not in the above lists then no support
            _ => None,
        }
    }

    async fn from_str(
        &self,
        str: &str,
        options: Option<DecodeOptions>,
    ) -> Result<(Node, Losses, Mapping)> {
        let (node, losses) = decode::decode(str, options)?;
        Ok((node, losses, Mapping::none()))
    }

    async fn to_string(
        &self,
        node: &Node,
        options: Option<EncodeOptions>,
    ) -> Result<(String, Losses, Mapping)> {
        encode::encode(node, options)
    }
}
