use std::path::Path;

use codec::{
    Codec, CodecAvailability, CodecSupport, DecodeInfo, DecodeOptions, EncodeInfo, EncodeOptions,
    async_trait, eyre::Result, format::Format, schema::Node, status::Status,
};
use codec_docx::DocxCodec;

/// A codec for DOCX compatible with uploads/downloads to/from Google Docs
pub struct GDocxCodec;

#[async_trait]
impl Codec for GDocxCodec {
    fn name(&self) -> &str {
        "gdocx"
    }

    fn status(&self) -> Status {
        Status::Alpha
    }

    fn availability(&self) -> CodecAvailability {
        DocxCodec.availability()
    }

    fn supports_from_format(&self, format: &Format) -> CodecSupport {
        match format {
            Format::GDocx => CodecSupport::LowLoss,
            _ => CodecSupport::None,
        }
    }

    fn supports_to_format(&self, format: &Format) -> CodecSupport {
        match format {
            Format::GDocx => CodecSupport::LowLoss,
            _ => CodecSupport::None,
        }
    }

    async fn from_path(
        &self,
        path: &Path,
        options: Option<DecodeOptions>,
    ) -> Result<(Node, Option<Node>, DecodeInfo)> {
        let options = DecodeOptions {
            format: Some(Format::GDocx),
            ..options.unwrap_or_default()
        };

        DocxCodec.from_path(path, Some(options)).await
    }

    async fn to_path(
        &self,
        node: &Node,
        path: &Path,
        options: Option<EncodeOptions>,
    ) -> Result<EncodeInfo> {
        let options = EncodeOptions {
            format: Some(Format::GDocx),
            ..options.unwrap_or_default()
        };

        DocxCodec.to_path(node, path, Some(options)).await
    }
}
