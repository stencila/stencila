use std::path::Path;

use codec_markdown::MarkdownCodec;
use codec_utils::git_info;
use convert::{latex_to_pdf, pdf_to_md};

use codec::{
    Codec, CodecSupport, DecodeInfo, DecodeOptions, EncodeInfo, EncodeOptions, NodeType,
    common::{async_trait::async_trait, eyre::Result},
    format::Format,
    schema::{Article, Node},
    status::Status,
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
        // Convert the PDF to a Markdown file
        let tool = options.as_ref().and_then(|opts| opts.tool.as_deref());
        let md_path = pdf_to_md(path, tool).await?;

        // Decode the Markdown file to a node
        let (mut node, orig, info) = MarkdownCodec.from_path(&md_path, options).await?;

        // Set source information, so that it refers to the PDF, not the temporary Markdown file
        if let Node::Article(Article { options, .. }) = &mut node {
            let git_info = git_info(path)?;
            options.repository = git_info.origin;
            options.path = git_info.path;
            options.commit = git_info.commit;
        }

        Ok((node, orig, info))
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
