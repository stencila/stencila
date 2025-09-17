use std::{env::current_dir, path::Path};

use tempfile::tempdir;
use tokio::fs::{read, write};

use stencila_codec::{
    Codec, CodecSupport, DecodeInfo, DecodeOptions, EncodeInfo, EncodeOptions, NodeType,
    StructuringOperation, StructuringOptions, async_trait,
    eyre::Result,
    stencila_format::Format,
    stencila_schema::{Article, File, InstructionMessage, MessagePart, ModelParameters, Node},
};
use stencila_codec_dom::DomCodec;
use stencila_codec_latex::LatexCodec;
use stencila_codec_markdown::MarkdownCodec;
use stencila_codec_utils::git_info;
use stencila_convert::{clean_md, html_to_pdf, latex_to_pdf};
use stencila_dirs::closest_artifacts_for;
use stencila_models::{ModelTask, perform_task};
use stencila_node_media::embed_media;
use stencila_schema_json::json_schema;

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

    fn structuring_options(&self, _format: &Format) -> StructuringOptions {
        use StructuringOperation::*;
        StructuringOptions::new([All], [RemovePrePrimary, ParagraphsToSentences])
    }

    async fn from_path(
        &self,
        path: &Path,
        options: Option<DecodeOptions>,
    ) -> Result<(Node, Option<Node>, DecodeInfo)> {
        let ignore_artifacts = options
            .as_ref()
            .and_then(|opts| opts.ignore_artifacts)
            .unwrap_or_default();
        let no_artifacts = options
            .as_ref()
            .and_then(|opts| opts.no_artifacts)
            .unwrap_or_default();

        // Read PDF
        let pdf_bytes = read(path).await?;

        // Create temporary directory (must be kept alive for entire function)
        let temp_dir = tempdir()?;

        // Determine where to store/look for artifacts
        let artifacts_path = if no_artifacts {
            // Don't cache, use temporary directory
            temp_dir.path().to_path_buf()
        } else {
            // Use artifacts directory for caching using PDF hash digest as key
            let digest = seahash::hash(&pdf_bytes);
            let key = format!("pdfmd-{digest:x}");
            closest_artifacts_for(&current_dir()?, &key).await?
        };

        let md_path = artifacts_path.join("intermediate.md");

        // Generate the Markdown document if needed
        let should_generate = !md_path.exists() || ignore_artifacts;
        if should_generate {
            let output = perform_task(ModelTask {
                // Specify schema for metadata extraction
                format: Some(Format::Json),
                schema: Some(json_schema("article:metadata")?),
                // Specify model
                model_parameters: Some(ModelParameters {
                    model_ids: Some(vec!["mistral/mistral-ocr-2505".to_string()]),
                    ..Default::default()
                }),
                // Include PDF in message
                messages: vec![InstructionMessage {
                    parts: vec![MessagePart::File(File::read(path)?)],
                    ..Default::default()
                }],
                ..Default::default()
            })
            .await?;

            // Clean the generated Markdown
            let markdown = clean_md(&output.content);

            // Write generated Markdown and any attachments into artifacts folder
            write(&md_path, &markdown).await?;
            for attachment in output.attachments {
                attachment.write(&artifacts_path.join(&attachment.name))?;
            }
        }

        // Decode the Markdown file to a node
        let (mut node, orig, info) = MarkdownCodec.from_path(&md_path, options).await?;

        // Embed any image files
        embed_media(&mut node, Some(&md_path))?;

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
                        // Paged view for CSS Paged Media support
                        view: Some("paged".to_string()),
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
