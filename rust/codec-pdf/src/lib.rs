use std::path::Path;

use codec_markdown::MarkdownCodec;
use convert::{latex_to_pdf, pdf_to_md};

use codec::{
    common::{async_trait::async_trait, eyre::Result},
    format::Format,
    schema::Node,
    status::Status,
    Codec, CodecSupport, DecodeInfo, DecodeOptions, EncodeInfo, EncodeOptions, NodeType,
};
use codec_latex::LatexCodec;
/// A codec for PDF
pub struct PdfCodec;

#[async_trait]
impl Codec for PdfCodec {
    fn name(&self) -> &str {
        "pdf"
    }

    fn status(&self) -> Status {
        Status::Beta
    }

    fn supports_from_format(&self, format: &Format) -> CodecSupport {
        match format {
            Format::Pdf => CodecSupport::HighLoss,
            _ => CodecSupport::None,
        }
    }

    fn supports_to_format(&self, format: &Format) -> CodecSupport {
        match format {
            Format::Pdf => CodecSupport::HighLoss,
            _ => CodecSupport::None,
        }
    }

    fn supports_from_type(&self, _node_type: NodeType) -> CodecSupport {
        CodecSupport::HighLoss
    }

    fn supports_to_type(&self, _node_type: NodeType) -> CodecSupport {
        CodecSupport::HighLoss
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
    ) -> Result<(Node, Option<Node>, DecodeInfo)> {
        let md_path = pdf_to_md(path).await?;

        MarkdownCodec.from_path(&md_path, options).await
    }

    async fn to_path(
        &self,
        node: &Node,
        path: &Path,
        options: Option<EncodeOptions>,
    ) -> Result<EncodeInfo> {
        let options = options.unwrap_or_default();

        let (latex, info) = LatexCodec
            .to_string(
                node,
                Some(EncodeOptions {
                    standalone: Some(true),
                    render: Some(true),
                    // Indicate that the LaTeX should be generated for PDF as final
                    // destination format
                    format: Some(Format::Pdf),
                    ..options
                }),
            )
            .await?;

        latex_to_pdf(&latex, path).await?;

        Ok(info)
    }
}
