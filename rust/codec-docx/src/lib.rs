use std::path::Path;

use codec::{
    common::{async_trait::async_trait, eyre::Result},
    format::Format,
    schema::Node,
    status::Status,
    Codec, CodecAvailability, CodecSupport, DecodeInfo, DecodeOptions, EncodeInfo, EncodeOptions,
    NodeType,
};
use codec_pandoc::{
    coarse_to_path, pandoc_availability, pandoc_from_format, pandoc_to_format, root_from_pandoc,
    root_to_pandoc,
};

/// A codec for Microsoft Word DOCX
pub struct DocxCodec;

const PANDOC_FORMAT: &str = "docx";

#[async_trait]
impl Codec for DocxCodec {
    fn name(&self) -> &str {
        "docx"
    }

    fn status(&self) -> Status {
        Status::UnderDevelopment
    }

    fn availability(&self) -> CodecAvailability {
        pandoc_availability()
    }

    fn supports_from_format(&self, format: &Format) -> CodecSupport {
        match format {
            Format::Docx => CodecSupport::LowLoss,
            _ => CodecSupport::None,
        }
    }

    fn supports_to_format(&self, format: &Format) -> CodecSupport {
        match format {
            Format::Docx => CodecSupport::LowLoss,
            _ => CodecSupport::None,
        }
    }

    fn supports_from_type(&self, _node_type: NodeType) -> CodecSupport {
        CodecSupport::LowLoss
    }

    fn supports_to_type(&self, _node_type: NodeType) -> CodecSupport {
        CodecSupport::LowLoss
    }

    fn supports_from_string(&self) -> bool {
        false
    }

    fn supports_to_string(&self) -> bool {
        false
    }

    async fn from_path(
        &self,
        path: &Path,
        options: Option<DecodeOptions>,
    ) -> Result<(Node, DecodeInfo)> {
        let pandoc = pandoc_from_format(
            "",
            Some(path),
            &[PANDOC_FORMAT, "+styles"].concat(),
            &options,
        )
        .await?;
        root_from_pandoc(pandoc, Format::Docx, &options)
    }

    async fn to_path(
        &self,
        node: &Node,
        path: &Path,
        options: Option<EncodeOptions>,
    ) -> Result<EncodeInfo> {
        let mut options = options.unwrap_or_default();
        if options.render.is_none() {
            options.render = Some(true);
        }

        if options.render.unwrap_or_default() {
            if let Node::Article(article) = node {
                if article.is_coarse(&Format::Latex) {
                    return coarse_to_path(node, Format::Latex, Format::Docx, path, Some(options))
                        .await;
                }
            }
        }

        let options = Some(options);
        let (pandoc, info) = root_to_pandoc(node, Format::Docx, &options)?;
        pandoc_to_format(
            &pandoc,
            Some(path),
            &[PANDOC_FORMAT, "+styles+native_numbering"].concat(),
            &options,
        )
        .await?;

        Ok(info)
    }
}
