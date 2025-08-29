use std::{env::current_dir, path::Path};

use codec::{
    Codec, CodecSupport, DecodeInfo, DecodeOptions, EncodeInfo, EncodeOptions,
    common::{
        async_trait::async_trait,
        eyre::Result,
        tokio::fs::{create_dir_all, write},
    },
    format::Format,
    schema::{Node, NodeType},
    status::Status,
};

pub use codec_markdown_trait::{to_markdown, to_markdown_flavor};

mod decode;
pub use decode::decode;
pub use decode::decode_frontmatter;
pub use decode::preprocess;

mod encode;
pub use encode::encode;
use media_embed::embed_media;
use media_extract::extract_media;

/// A codec for Markdown
pub struct MarkdownCodec;

#[async_trait]
impl Codec for MarkdownCodec {
    fn name(&self) -> &str {
        "markdown"
    }

    fn status(&self) -> Status {
        Status::Beta
    }

    fn supports_from_format(&self, format: &Format) -> CodecSupport {
        use CodecSupport::*;
        match format {
            Format::Markdown | Format::Smd | Format::Myst | Format::Qmd => LowLoss,
            _ => None,
        }
    }

    fn supports_to_format(&self, format: &Format) -> CodecSupport {
        use CodecSupport::*;
        match format {
            Format::Markdown | Format::Smd | Format::Myst | Format::Qmd | Format::Llmd => LowLoss,
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

    async fn from_str(
        &self,
        str: &str,
        options: Option<DecodeOptions>,
    ) -> Result<(Node, DecodeInfo)> {
        decode(str, options)
    }

    async fn to_string(
        &self,
        node: &Node,
        options: Option<EncodeOptions>,
    ) -> Result<(String, EncodeInfo)> {
        encode(node, options)
    }

    async fn to_path(
        &self,
        node: &Node,
        path: &Path,
        options: Option<EncodeOptions>,
    ) -> Result<EncodeInfo> {
        let mut options = options.unwrap_or_default();
        if options.standalone.is_none() {
            options.standalone = Some(true);
        }
        if !options.embed_media.unwrap_or_default() && options.extract_media.is_none() {
            options.extract_media = Some(path.with_extension("media"));
        }
        options.to_path = Some(path.to_path_buf());

        let (md, info) = if let Some(media) = &options.extract_media {
            let mut copy = node.clone();
            extract_media(&mut copy, media)?;
            encode(&copy, Some(options))?
        } else if options.embed_media.unwrap_or_default() {
            let mut copy = node.clone();
            let from_path = match &options.from_path {
                Some(path) => path.clone(),
                None => current_dir()?,
            };
            embed_media(&mut copy, &from_path)?;
            encode(&copy, Some(options))?
        } else {
            encode(node, Some(options))?
        };

        if let Some(parent) = path.parent() {
            create_dir_all(parent).await?;
        }
        write(&path, md).await?;

        Ok(info)
    }
}
