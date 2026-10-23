use std::path::Path;

use stencila_codec::{
    Codec, DecodeInfo, DecodeOptions, EncodeInfo, EncodeOptions, StructuringOptions, async_trait,
    eyre::Result, stencila_format::Format, stencila_schema::Node,
};

mod decode;
mod encode;
mod losses;

pub use decode::decode;
pub use encode::encode;
pub use losses::{LossCategory, classify};

#[cfg(test)]
mod tests;

/// A codec for JATS
pub struct JatsCodec;

#[async_trait]
impl Codec for JatsCodec {
    fn name(&self) -> &str {
        "jats"
    }

    fn supports_from_format(&self, format: &Format) -> bool {
        // Publishers and PMC use plain .xml and .nxml filenames as well as
        // .jats.xml. Decoding still requires an <article> element.
        matches!(format, Format::Jats)
            || matches!(format, Format::Other(name) if matches!(name.as_str(), "xml" | "nxml"))
    }

    fn supports_to_format(&self, format: &Format) -> bool {
        matches!(format, Format::Jats)
    }

    fn structuring_options(&self, _format: &Format) -> StructuringOptions {
        use stencila_codec::StructuringOperation::*;
        StructuringOptions::new([NormalizeCitations], [])
    }

    async fn from_str(
        &self,
        str: &str,
        options: Option<DecodeOptions>,
    ) -> Result<(Node, DecodeInfo)> {
        decode(str, options)
    }

    async fn from_path(
        &self,
        path: &Path,
        options: Option<DecodeOptions>,
    ) -> Result<(Node, Option<Node>, DecodeInfo)> {
        let path = path.canonicalize()?;
        let content = std::fs::read_to_string(&path)?;
        let (mut node, info) = decode::decode_with_directory(&content, path.parent())?;
        let options = options.unwrap_or_default();

        if options.reproducible.unwrap_or(true)
            && let Node::Article(article) = &mut node
        {
            let git = stencila_codec_utils::git_file_info(&path)?;
            article.options.repository = git.origin;
            article.options.path = git.path;
            article.options.commit = git.commit;
        }
        if options.embed_media.unwrap_or_default() {
            stencila_node_media::embed_media(&mut node, path.parent())?;
        }
        if options.embed_supplements.unwrap_or_default() {
            stencila_node_supplements::embed_supplements(&mut node, &path).await?;
        }
        Ok((node, None, info))
    }

    async fn to_string(
        &self,
        node: &Node,
        options: Option<EncodeOptions>,
    ) -> Result<(String, EncodeInfo)> {
        encode(node, options)
    }
}
