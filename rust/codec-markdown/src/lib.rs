use std::path::Path;

use tokio::fs::{create_dir_all, write};

use stencila_codec::{
    Codec, DecodeInfo, DecodeOptions, EncodeInfo, EncodeOptions, StructuringOperation,
    StructuringOptions, async_trait, eyre::Result, stencila_format::Format, stencila_schema::Node,
};

pub use stencila_codec_markdown_trait::{to_markdown, to_markdown_flavor};

mod decode;
pub use decode::decode;
pub use decode::decode_frontmatter;
pub use decode::preprocess;

mod encode;
pub use encode::encode;
use stencila_node_media::{embed_media, extract_media_with_paths};

/// A codec for Markdown
pub struct MarkdownCodec;

#[async_trait]
impl Codec for MarkdownCodec {
    fn name(&self) -> &str {
        "markdown"
    }

    fn supports_from_format(&self, format: &Format) -> bool {
        matches!(
            format,
            Format::Markdown | Format::Smd | Format::Myst | Format::Qmd
        )
    }

    fn supports_to_format(&self, format: &Format) -> bool {
        matches!(
            format,
            Format::Markdown | Format::Smd | Format::Myst | Format::Qmd | Format::Llmd
        )
    }

    fn structuring_options(&self, _format: &Format) -> StructuringOptions {
        use StructuringOperation::*;
        StructuringOptions::new(
            [
                Heading1ToTitleSingle,
                HeadingsDecrement,
                FiguresWithCaptions,
                TablesWithCaptions,
                TablesToDatatables,
                TextToLinks,
            ],
            [],
        )
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
        if let Some(media) = options
            .as_ref()
            .and_then(|opts| opts.extract_media.as_ref())
        {
            let mut copy = node.clone();
            let assets = extract_media_with_paths(
                &mut copy,
                options.as_ref().and_then(|opts| opts.to_path.as_deref()),
                media,
            )?;
            let (md, mut info) = encode(&copy, options)?;
            info.assets.extend(assets);
            Ok((md, info))
        } else if options
            .as_ref()
            .and_then(|opts| opts.embed_media)
            .unwrap_or_default()
        {
            let mut copy = node.clone();
            embed_media(
                &mut copy,
                options.as_ref().and_then(|opts| opts.from_path.as_deref()),
            )?;
            encode(&copy, options)
        } else {
            encode(node, options)
        }
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

        let (md, info) = self.to_string(node, Some(options)).await?;

        if let Some(parent) = path.parent() {
            create_dir_all(parent).await?;
        }
        write(&path, md).await?;

        Ok(info)
    }
}
