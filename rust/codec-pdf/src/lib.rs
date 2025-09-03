use std::path::Path;

use stencila_codec::{
    Codec, CodecSupport, DecodeInfo, DecodeOptions, EncodeInfo, EncodeOptions, NodeType,
    async_trait,
    eyre::Result,
    stencila_format::Format,
    stencila_schema::{Article, Node},
};
use stencila_codec_dom::DomCodec;
use stencila_codec_latex::LatexCodec;
use stencila_codec_markdown::MarkdownCodec;
use stencila_codec_utils::git_info;
use stencila_convert::{html_to_pdf, latex_to_pdf, pdf_to_md};
use stencila_media_embed::embed_media;
use stencila_node_structuring::structuring;

/// A codec for PDF
pub struct PdfCodec;

#[async_trait]
impl Codec for PdfCodec {
    fn name(&self) -> &str {
        "pdf"
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
        let include_pages = options
            .as_ref()
            .and_then(|opts| opts.include_pages.as_ref());
        let exclude_pages = options
            .as_ref()
            .and_then(|opts| opts.exclude_pages.as_ref());
        let ignore_artifacts = options
            .as_ref()
            .and_then(|opts| opts.ignore_artifacts)
            .unwrap_or_default();
        let no_artifacts = options
            .as_ref()
            .and_then(|opts| opts.no_artifacts)
            .unwrap_or_default();
        let md_path = pdf_to_md(
            path,
            tool,
            include_pages,
            exclude_pages,
            ignore_artifacts,
            no_artifacts,
        )
        .await?;

        // Decode the Markdown file to a node
        let (mut node, orig, info) = MarkdownCodec.from_path(&md_path, options).await?;

        // Increase structure of the node
        structuring(&mut node);

        // Embed any image files
        embed_media(&mut node, &md_path)?;

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

        let tool = options.tool.as_deref().unwrap_or("browser");
        if matches!(tool, "xelatex" | "latex") {
            // Encode to PDF via LaTeX
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
        } else {
            // Encode to PDF via HTML
            let (html, info) = DomCodec
                .to_string(
                    node,
                    Some(EncodeOptions {
                        // Standalone so that necessary JS and CSS is loaded
                        standalone: Some(true),
                        // Embed any media files. This is necessary because the
                        // browser will not fetch local resources when generating
                        // the PDF
                        embed_media: Some(true),
                        ..options
                    }),
                )
                .await?;

            html_to_pdf(&html, path)?;

            Ok(info)
        }
    }
}
