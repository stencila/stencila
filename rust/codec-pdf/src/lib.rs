use std::{env::current_dir, path::Path};

use tempfile::tempdir;
use tokio::fs::{read, read_to_string, write};

use stencila_codec::{
    Codec, CodecSupport, DecodeInfo, DecodeOptions, EncodeInfo, EncodeOptions, NodeType,
    StructuringOperation, StructuringOptions, async_trait,
    eyre::Result,
    stencila_format::Format,
    stencila_schema::{Article, File, MessagePart, ModelParameters, Node},
};
use stencila_codec_dom::DomCodec;
use stencila_codec_latex::LatexCodec;
use stencila_codec_markdown::MarkdownCodec;
use stencila_codec_utils::git_file_info;
use stencila_convert::{clean_md, html_to_pdf, latex_to_pdf};
use stencila_dirs::closest_artifacts_for;
use stencila_models::{ModelMessage, ModelTask, perform_task};
use stencila_node_media::embed_media;
use stencila_schema_json::{JsonSchemaVariant, json_schema};

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
        StructuringOptions::new(
            [All],
            [
                HeadingsToParagraphs,
                RemovePrePrimary,
                ParagraphsToSentences,
            ],
        )
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
        tracing::trace!("Reading PDF");
        let pdf_bytes = read(path).await?;

        // Load PDF to get page count
        let pdf_doc = lopdf::Document::load_mem(&pdf_bytes)?;
        let page_count = pdf_doc.get_pages().len() as u32;

        // Create temporary directory (must be kept alive for entire function)
        let temp_dir = tempdir()?;

        // Determine where to store/look for artifacts
        let artifacts_path = if no_artifacts {
            // Don't cache, use temporary directory
            temp_dir.path().to_path_buf()
        } else {
            // Use artifacts directory for caching using PDF hash digest as key
            tracing::trace!("Hashing PDF");
            let digest = seahash::hash(&pdf_bytes);
            let key = format!("pdfmd-{digest:x}");
            closest_artifacts_for(&current_dir()?, &key).await?
        };

        let ocr_md_path = artifacts_path.join("ocr.md");
        let clean_md_path = artifacts_path.join("clean.md");

        // Generate the Markdown document if needed
        let should_generate = !ocr_md_path.exists() || ignore_artifacts;
        let ocr_md = if should_generate {
            let model_parameters = Some(ModelParameters {
                model_ids: Some(vec!["mistral/mistral-ocr-2505".to_string()]),
                ..Default::default()
            });

            // Maximum number of pages for PDFs when using schema-based metadata extraction
            // This is a limitation of Mistral OCR
            const MAX_PAGES_WITH_SCHEMA: u32 = 8;
            let ocr_md = if page_count <= MAX_PAGES_WITH_SCHEMA {
                // Small PDF: single pass with metadata extraction
                let output = perform_task(ModelTask {
                    format: Some(Format::Markdown),
                    // Specify schema for metadata extraction
                    schema: Some(json_schema(JsonSchemaVariant::ArticleMetadata)),
                    // Specify model
                    model_parameters,
                    // Include PDF in message
                    messages: vec![ModelMessage::system(vec![MessagePart::File(File::read(
                        path,
                    )?)])],
                    ..Default::default()
                })
                .await?;

                // Write attachments
                for attachment in output.attachments {
                    attachment.write(&artifacts_path.join(&attachment.name))?;
                }

                output.content
            } else {
                // Large PDF: extract first MAX_PAGES_WITH_SCHEMA pages for metadata, process full PDF for content

                // Extract start pages by creating a new document and deleting pages MAX_PAGES_WITH_SCHEMA+
                let mut partial_doc = lopdf::Document::load_mem(&pdf_bytes)?;
                let pages_to_delete: Vec<u32> =
                    (MAX_PAGES_WITH_SCHEMA.saturating_add(1)..=page_count).collect();
                partial_doc.delete_pages(&pages_to_delete);
                partial_doc.prune_objects();

                // Write partial PDF to temp file
                let partial_pdf_path = temp_dir.path().join("partial.pdf");
                partial_doc.save(&partial_pdf_path)?;

                // Step 1: Get metadata from first pages WITH schema
                let metadata_output = perform_task(ModelTask {
                    format: Some(Format::Markdown),
                    schema: Some(json_schema(JsonSchemaVariant::ArticleMetadata)),
                    model_parameters: model_parameters.clone(),
                    messages: vec![ModelMessage::system(vec![MessagePart::File(File::read(
                        &partial_pdf_path,
                    )?)])],
                    ..Default::default()
                })
                .await?;

                // Step 2: Get full content from entire PDF WITHOUT schema
                let content_output = perform_task(ModelTask {
                    format: Some(Format::Markdown),
                    // No schema for full content
                    schema: None,
                    model_parameters,
                    messages: vec![ModelMessage::system(vec![MessagePart::File(File::read(
                        path,
                    )?)])],
                    ..Default::default()
                })
                .await?;

                // Write attachments from full content only
                for attachment in content_output.attachments {
                    attachment.write(&artifacts_path.join(&attachment.name))?;
                }

                // Step 3: Extract YAML front matter from metadata output
                let metadata_md = &metadata_output.content;
                let front_matter = if let Some(first_delim) = metadata_md.find("---") {
                    if let Some(second_delim) = metadata_md[first_delim + 3..].find("---") {
                        let end = first_delim + 3 + second_delim + 3;
                        &metadata_md[..end]
                    } else {
                        ""
                    }
                } else {
                    ""
                };

                // Combine front matter with full content
                if !front_matter.is_empty() {
                    format!("{}\n\n{}", front_matter, content_output.content)
                } else {
                    content_output.content
                }
            };

            // Write raw OCR markdown
            write(&ocr_md_path, &ocr_md).await?;

            ocr_md
        } else {
            read_to_string(&ocr_md_path).await?
        };

        // Clean and write cleaned Markdown
        tracing::trace!("Cleaning Markdown");
        let clean_md = clean_md(&ocr_md);
        write(&clean_md_path, &clean_md).await?;

        // Decode the Markdown file to a node
        tracing::trace!("Decoding Markdown");
        let (mut node, orig, info) = MarkdownCodec.from_path(&clean_md_path, options).await?;

        // Embed any image files
        tracing::trace!("Embedding media");
        embed_media(&mut node, Some(&clean_md_path))?;

        // Set source information, so that it refers to the PDF, not the temporary Markdown file
        if let Node::Article(Article { options, .. }) = &mut node {
            let git_file_info = git_file_info(path)?;
            options.repository = git_file_info.origin;
            options.path = git_file_info.path;
            options.commit = git_file_info.commit;
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
                        // Use static view in support of syntax highlighting,
                        // visualizations etc
                        view: Some("static".into()),
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
