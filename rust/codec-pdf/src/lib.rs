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
    pandoc_availability, pandoc_from_format, pandoc_to_format, root_from_pandoc, root_to_pandoc,
};

/// A codec for PDF
pub struct PdfCodec;

const PANDOC_FORMAT: &str = "pdf";

#[async_trait]
impl Codec for PdfCodec {
    fn name(&self) -> &str {
        "pdf"
    }

    fn status(&self) -> Status {
        Status::UnderDevelopment
    }

    fn availability(&self) -> CodecAvailability {
        pandoc_availability()
    }

    fn supports_from_format(&self, _format: &Format) -> CodecSupport {
        CodecSupport::None
    }

    fn supports_to_format(&self, format: &Format) -> CodecSupport {
        match format {
            Format::Pdf => CodecSupport::LowLoss,
            _ => CodecSupport::None,
        }
    }

    fn supports_from_type(&self, _node_type: NodeType) -> CodecSupport {
        CodecSupport::None
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
            PANDOC_FORMAT,
            options
                .map(|options| options.passthrough_args)
                .unwrap_or_default(),
        )
        .await?;
        root_from_pandoc(pandoc, Format::Pdf)
    }

    async fn to_path(
        &self,
        node: &Node,
        path: &Path,
        options: Option<EncodeOptions>,
    ) -> Result<EncodeInfo> {
        let (pandoc, info) = root_to_pandoc(node, Format::Pdf)?;
        pandoc_to_format(
            &pandoc,
            Some(path),
            PANDOC_FORMAT,
            options
                .map(|options| options.passthrough_args)
                .unwrap_or_default(),
        )
        .await?;
        Ok(info)
    }
}
