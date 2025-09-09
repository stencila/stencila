use std::path::{Path, PathBuf};

use eyre::{Context, Result, bail, eyre};

use stencila_codec::{Codec, stencila_format::Format};
use stencila_codec_csv::CsvCodec;
use stencila_codec_docx::DocxCodec;
use stencila_codec_ipynb::IpynbCodec;
use stencila_codec_latex::LatexCodec;
use stencila_codec_pdf::PdfCodec;
use stencila_codec_xlsx::XlsxCodec;
use stencila_node_media::embed_media;
use stencila_schema::{
    Article, AudioObject, Block, CompilationMessage, CreativeWorkType, CreativeWorkVariant, Figure,
    File, ImageObject, MessageLevel, Node, Supplement, VideoObject, VisitorAsync, WalkControl,
    WalkNode,
};

/// Embed any [`Supplement`] nodes within a node
pub async fn embed_supplements<T: WalkNode>(node: &mut T, path: &Path) -> Result<()> {
    let path = path
        .canonicalize()
        .wrap_err_with(|| eyre!("Path does not exist: {}", path.display()))?;

    let dir = if path.is_file()
        && let Some(parent) = path.parent()
    {
        parent.to_path_buf()
    } else {
        path
    };

    if !dir.exists() {
        bail!("Directory does not exist: {}", dir.display());
    }

    let mut embedder = Embedder { dir };
    node.walk_async(&mut embedder).await?;

    Ok(())
}

struct Embedder {
    /// The directory from which relative paths to supplements are resolved
    dir: PathBuf,
}

impl VisitorAsync for Embedder {
    async fn visit_node(&mut self, node: &mut Node) -> Result<WalkControl> {
        if let Node::Supplement(supplement) = node {
            self.embed(supplement).await;
            Ok(WalkControl::Break)
        } else {
            Ok(WalkControl::Continue)
        }
    }

    async fn visit_block(&mut self, block: &mut Block) -> Result<WalkControl> {
        if let Block::Supplement(supplement) = block {
            self.embed(supplement).await;
            Ok(WalkControl::Break)
        } else {
            Ok(WalkControl::Continue)
        }
    }
}

impl Embedder {
    /// Embed a supplement by resolving its target file and converting it to appropriate content.
    ///
    /// Validates the supplement has a target, resolves the file path, determines the format,
    /// and processes content accordingly - creating media objects for audio/video/images or
    /// using codecs to decode structured documents (CSV, DOCX, etc.). Handles errors gracefully
    /// by recording compilation messages and maps the result to the correct work type and variant.
    async fn embed(&self, supplement: &mut Supplement) {
        let Some(target) = &supplement.target else {
            supplement.options.compilation_messages = Some(vec![CompilationMessage::new(
                MessageLevel::Warning,
                "Supplement has no link target to embed".into(),
            )]);

            return;
        };

        // Resolve the target
        let target = PathBuf::from(target);
        let path = if target.is_absolute() {
            target
        } else {
            self.dir.join(target)
        };

        // Canonicalize the path for `File` variant
        let path = match path.canonicalize() {
            Ok(path) => path,
            Err(..) => {
                supplement.options.compilation_messages = Some(vec![CompilationMessage::new(
                    MessageLevel::Error,
                    format!("Supplement link target does not exist: {}", path.display()),
                )]);

                return;
            }
        };

        // Determine format of the supplement from the path
        let format = Format::from_path(&path);

        let node = if format.is_media() {
            let content_url = path.to_string_lossy().to_string();

            let mut node = if format.is_audio() {
                Node::AudioObject(AudioObject::new(content_url))
            } else if format.is_video() {
                Node::VideoObject(VideoObject::new(content_url))
            } else {
                Node::ImageObject(ImageObject::new(content_url))
            };

            if let Err(error) = embed_media(&mut node, &self.dir) {
                supplement.options.compilation_messages = Some(vec![CompilationMessage::new(
                    MessageLevel::Error,
                    format!(
                        "While embedding supplement `{}` media: {error}",
                        path.display()
                    ),
                )]);
            }

            if matches!(supplement.work_type, Some(CreativeWorkType::Figure)) {
                Node::Figure(Figure {
                    content: vec![node.try_into().expect("media types can be blocks")],
                    ..Default::default()
                })
            } else {
                node
            }
        } else {
            // Decode the path
            let result = match format {
                Format::Csv | Format::Tsv => CsvCodec.from_path(&path, None),
                Format::Docx => DocxCodec.from_path(&path, None),
                Format::Ipynb => IpynbCodec.from_path(&path, None),
                Format::Latex => LatexCodec.from_path(&path, None),
                Format::Pdf => PdfCodec.from_path(&path, None),
                Format::Xlsx | Format::Xls => XlsxCodec.from_path(&path, None),
                _ => {
                    let name = path.file_name().map_or_else(
                        || "Unnamed".to_string(),
                        |name| name.to_string_lossy().to_string(),
                    );
                    let path = path.to_string_lossy().to_string();
                    let file = CreativeWorkVariant::File(File::new(name, path));

                    supplement.options.work = Some(file);

                    return;
                }
            }
            .await;

            // Record any error as a compilation message
            let mut node = match result {
                Ok((node, ..)) => node,
                Err(error) => {
                    supplement.options.compilation_messages = Some(vec![CompilationMessage::new(
                        MessageLevel::Error,
                        format!("While decoding supplement: {error}"),
                    )]);

                    return;
                }
            };

            // If the decoded node is an article, it has only one node, and that
            // node is consistent the expected work type then use that inner
            // node (e.g. a docx with a table or image + caption in it)
            if let Some(work_type) = supplement.work_type
                && let Node::Article(Article { content, .. }) = &node
                && content.len() == 1
                && let Some(block) = content.first()
            {
                match (work_type, block) {
                    (CreativeWorkType::Table, Block::Table(table)) => {
                        node = Node::Table(table.clone());
                    }
                    (CreativeWorkType::Table, Block::Datatable(table)) => {
                        node = Node::Datatable(table.clone());
                    }
                    (CreativeWorkType::Datatable, Block::Datatable(table)) => {
                        node = Node::Datatable(table.clone());
                    }
                    _ => {}
                }
            }

            node
        };

        let (work_type, work) = match node {
            Node::Article(article) => (
                CreativeWorkType::Article,
                CreativeWorkVariant::Article(article),
            ),
            Node::Figure(figure) => (
                CreativeWorkType::Figure,
                CreativeWorkVariant::Figure(figure),
            ),
            Node::Table(table) => (CreativeWorkType::Table, CreativeWorkVariant::Table(table)),
            Node::Datatable(datatable) => (
                CreativeWorkType::Datatable,
                CreativeWorkVariant::Datatable(datatable),
            ),
            Node::AudioObject(audio) => (
                CreativeWorkType::AudioObject,
                CreativeWorkVariant::AudioObject(audio),
            ),
            Node::ImageObject(image) => (
                CreativeWorkType::ImageObject,
                CreativeWorkVariant::ImageObject(image),
            ),
            Node::VideoObject(video) => (
                CreativeWorkType::VideoObject,
                CreativeWorkVariant::VideoObject(video),
            ),
            _ => return,
        };

        if supplement.work_type.is_none() {
            supplement.work_type = Some(work_type);
        }
        supplement.options.work = Some(work);
    }
}
