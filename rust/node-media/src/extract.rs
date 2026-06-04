use std::{
    env::current_dir,
    path::{Path, PathBuf},
};

use base64::{Engine as _, engine::general_purpose::STANDARD};
use eyre::{OptionExt, Result, bail, eyre};
use pathdiff::diff_paths;
use percent_encoding::percent_decode;

use stencila_codec_info::EncodedAsset;
use stencila_format::Format;
use stencila_schema::{
    AudioObject, Block, CodeChunk, CreativeWorkVariant, Figure, ImageObject, Inline, Node, NodeId,
    NodeType, VideoObject, VisitorMut, WalkControl, WalkNode,
};

use crate::naming::{MediaNamer, hash_bytes};

/// Extract any [`ImageObject`], [`AudioObject`], and [`VideoObject`] with
/// dataURIs to files and change their content_url to point to the extracted
/// files
///
/// This function processes all media objects in the document tree, extracting
/// embedded data URIs to the specified directory and updating the objects to
/// reference the extracted files instead.
///
/// See the `media-embed` crate for doing the opposite: embedding files as
/// dataURIs.
pub fn extract_media<T>(node: &mut T, document_path: Option<&Path>, media_dir: &Path) -> Result<()>
where
    T: WalkNode,
{
    extract_media_with_paths(node, document_path, media_dir).map(|_| ())
}

/// Extract media and return a record per asset written.
///
/// Each [`EncodedAsset`] is annotated with the originating node's id/type and
/// an asset role (e.g. `computational-output`, `math-image`, `table-image`,
/// `figure`) so dispatchers can attach per-node provenance to the file.
pub fn extract_media_with_paths<T>(
    node: &mut T,
    document_path: Option<&Path>,
    media_dir: &Path,
) -> Result<Vec<EncodedAsset>>
where
    T: WalkNode,
{
    // Determine the document directory (base for relative paths)
    let document_dir = match document_path {
        Some(path) => {
            // Get parent directory of the document file
            match path.parent() {
                Some(parent) if !parent.as_os_str().is_empty() => parent.to_path_buf(),
                _ => PathBuf::from("."),
            }
        }
        None => current_dir()?,
    };

    let mut walker = Extractor {
        document_dir,
        media_dir: media_dir.into(),
        parent_stack: Vec::new(),
        namer: MediaNamer::new(),
        assets: Vec::new(),
    };
    walker.walk(node);

    Ok(walker.assets)
}

struct Extractor {
    /// The directory containing the document. Used as base for relative paths to extracted media.
    document_dir: PathBuf,

    /// The directory where media files will be written
    media_dir: PathBuf,

    /// Stack of ancestor structs, used to attribute extracted assets to the
    /// closest meaningful originating node (executable, math/table container,
    /// or the media object itself).
    parent_stack: Vec<(NodeType, NodeId)>,

    /// State used to derive readable media filenames from nearby node ids.
    namer: MediaNamer,

    /// The asset records produced during extraction.
    assets: Vec<EncodedAsset>,
}

impl Extractor {
    /// Convert a data URI into a media file
    ///
    /// The media will be converted into a file with a name based on the hash of the
    /// URI and an extension based on the MIME type of the data URI.
    ///
    /// Returns the absolute path of the created media file and the relative
    /// path to use as the rewritten `content_url`.
    fn data_uri_to_file(
        &mut self,
        data_uri: &str,
        desired_stem: Option<&str>,
    ) -> Result<(PathBuf, String)> {
        // Parse the data URI
        let Some((header, data)) = data_uri.split_once(',') else {
            bail!("Invalid data URI format");
        };

        // Extract the MIME type
        let mime_type = header
            .split(';')
            .next()
            .and_then(|mime_type| mime_type.strip_prefix("data:"))
            .ok_or_eyre("Invalid data URI header")?;

        // Determine the format and extension from the MIME type
        let format = Format::from_media_type(mime_type)
            .map_err(|_| eyre!("Unsupported media format: {mime_type}"))?;

        let extension = if mime_type == "audio/mp4" {
            // Special case: audio/mp4 should use m4a extension
            "m4a".to_string()
        } else {
            format.extension()
        };

        let decoded_data = if header
            .split(';')
            .skip(1)
            .any(|parameter| parameter.eq_ignore_ascii_case("base64"))
        {
            STANDARD.decode(data.as_bytes())?
        } else {
            percent_decode(data.as_bytes()).collect()
        };

        let hash = hash_bytes(data_uri.as_bytes());
        let path = self.namer.write_bytes(
            &self.media_dir,
            desired_stem,
            &extension,
            hash,
            &decoded_data,
        )?;

        let relative_path = diff_paths(&path, &self.document_dir)
            .unwrap_or_else(|| path.clone())
            .to_string_lossy()
            .to_string();

        Ok((path, relative_path))
    }

    /// Record an extracted asset with originating-node attribution.
    fn record_asset(
        &mut self,
        path: PathBuf,
        self_id: Option<&NodeId>,
        self_type: NodeType,
        title: Option<String>,
        description: Option<String>,
    ) {
        let (node_type, node_id) = self.originating(self_id, self_type);
        let role = role_for(node_type);
        self.assets.push(EncodedAsset {
            path,
            node_id: node_id.map(|id| id.to_string()),
            node_type: Some(node_type.to_string()),
            role: Some(role.to_string()),
            title,
            description,
            ..Default::default()
        });
    }

    /// Compute the originating node for an asset.
    ///
    /// Prefers the closest executable ancestor (CodeChunk, CodeExpression,
    /// etc.), then any other meaningful container (MathBlock, MathInline,
    /// Table). Falls back to the media object itself.
    fn originating(
        &self,
        self_id: Option<&NodeId>,
        self_type: NodeType,
    ) -> (NodeType, Option<NodeId>) {
        if let Some((node_type, node_id)) = self
            .parent_stack
            .iter()
            .rev()
            .find(|(node_type, _)| is_executable(*node_type))
        {
            return (*node_type, Some(node_id.clone()));
        }

        if let Some((node_type, node_id)) = self
            .parent_stack
            .iter()
            .rev()
            .find(|(node_type, _)| is_media_container(*node_type))
        {
            return (*node_type, Some(node_id.clone()));
        }

        (self_type, self_id.cloned())
    }

    fn visit_figure(&mut self, figure: &mut Figure) {
        self.parent_stack.push((NodeType::Figure, figure.node_id()));

        if let Some(caption) = &mut figure.caption {
            caption.walk_mut(self);
        }

        self.namer.push_figure(figure);
        figure.content.walk_mut(self);
        self.namer.pop();

        self.parent_stack.pop();
    }

    fn visit_code_chunk(&mut self, chunk: &mut CodeChunk) {
        self.parent_stack
            .push((NodeType::CodeChunk, chunk.node_id()));

        if let Some(caption) = &mut chunk.caption {
            caption.walk_mut(self);
        }

        self.namer.push_code_chunk(chunk);
        chunk.outputs.walk_mut(self);
        self.namer.pop();

        self.parent_stack.pop();
    }

    fn extract_images(&mut self, images: &mut [ImageObject]) {
        images
            .iter_mut()
            .for_each(|image| self.extract_image(image));
    }

    fn extract_image(&mut self, image: &mut ImageObject) {
        if image.content_url.starts_with("data:") {
            let desired_stem = self.namer.next_media_stem(image.id.as_deref());
            let title = self.namer.next_media_title(image.title.as_deref());
            let description = self.namer.next_media_description(image.title.as_deref());
            match self.data_uri_to_file(&image.content_url, desired_stem.as_deref()) {
                Ok((path, file_path)) => {
                    let id = image.node_id();
                    self.record_asset(path, Some(&id), NodeType::ImageObject, title, description);
                    image.content_url = file_path;
                }
                Err(error) => tracing::error!("While writing image to file: {error}"),
            }
        }
    }

    fn extract_audio(&mut self, audio: &mut AudioObject) {
        if audio.content_url.starts_with("data:") {
            let desired_stem = self.namer.next_media_stem(audio.id.as_deref());
            let title = self.namer.next_media_title(audio.title.as_deref());
            let description = self.namer.next_media_description(audio.title.as_deref());
            match self.data_uri_to_file(&audio.content_url, desired_stem.as_deref()) {
                Ok((path, file_path)) => {
                    let id = audio.node_id();
                    self.record_asset(path, Some(&id), NodeType::AudioObject, title, description);
                    audio.content_url = file_path;
                }
                Err(error) => tracing::error!("While writing audio to file: {error}"),
            }
        }
    }

    fn extract_video(&mut self, video: &mut VideoObject) {
        if video.content_url.starts_with("data:") {
            let desired_stem = self.namer.next_media_stem(video.id.as_deref());
            let title = self.namer.next_media_title(video.title.as_deref());
            let description = self.namer.next_media_description(video.title.as_deref());
            match self.data_uri_to_file(&video.content_url, desired_stem.as_deref()) {
                Ok((path, file_path)) => {
                    let id = video.node_id();
                    self.record_asset(path, Some(&id), NodeType::VideoObject, title, description);
                    video.content_url = file_path;
                }
                Err(error) => tracing::error!("While writing video to file: {error}"),
            }
        }
    }
}

/// Node types that produce media as a side-effect of execution. Extracted
/// media inside these is attributed to the executable so per-asset
/// credentials carry that node's execution facts.
///
/// `Article` is intentionally excluded: a plain image in article body is
/// attributed to the image itself, not the article, so the per-asset
/// snapshot doesn't duplicate the document-level snapshot.
fn is_executable(node_type: NodeType) -> bool {
    matches!(
        node_type,
        NodeType::Button
            | NodeType::CallBlock
            | NodeType::CodeChunk
            | NodeType::CodeExpression
            | NodeType::ForBlock
            | NodeType::Form
            | NodeType::IfBlock
            | NodeType::IfBlockClause
            | NodeType::IncludeBlock
            | NodeType::InstructionBlock
            | NodeType::InstructionInline
            | NodeType::Parameter
            | NodeType::PromptBlock
    )
}

/// Non-executable container types whose own identity is the right credential
/// subject for media they wrap (rendered math, table images, etc.).
fn is_media_container(node_type: NodeType) -> bool {
    matches!(
        node_type,
        NodeType::MathBlock | NodeType::MathInline | NodeType::Table | NodeType::Figure
    )
}

/// Asset role string derived from the originating node type.
fn role_for(node_type: NodeType) -> &'static str {
    if is_executable(node_type) {
        return "computational-output";
    }
    match node_type {
        NodeType::MathBlock | NodeType::MathInline => "math-image",
        NodeType::Table => "table-image",
        NodeType::Figure => "figure",
        _ => "figure",
    }
}

impl VisitorMut for Extractor {
    fn enter_struct(&mut self, node_type: NodeType, node_id: NodeId) -> WalkControl {
        self.parent_stack.push((node_type, node_id));
        WalkControl::Continue
    }

    fn exit_struct(&mut self) {
        self.parent_stack.pop();
    }

    fn visit_node(&mut self, node: &mut Node) -> WalkControl {
        match node {
            Node::CodeChunk(chunk) => {
                self.visit_code_chunk(chunk);
                return WalkControl::Break;
            }
            Node::Figure(figure) => {
                self.visit_figure(figure);
                return WalkControl::Break;
            }
            Node::AudioObject(audio) => self.extract_audio(audio),
            Node::ImageObject(image) => self.extract_image(image),
            Node::VideoObject(video) => self.extract_video(video),
            Node::MathBlock(math) => {
                if let Some(images) = &mut math.options.images {
                    self.extract_images(images)
                }
            }
            Node::MathInline(math) => {
                if let Some(images) = &mut math.options.images {
                    self.extract_images(images)
                }
            }
            Node::Table(table) => {
                if let Some(images) = &mut table.options.images {
                    self.extract_images(images)
                }
            }
            _ => {}
        }
        WalkControl::Continue
    }

    fn visit_work(&mut self, work: &mut CreativeWorkVariant) -> WalkControl {
        match work {
            CreativeWorkVariant::AudioObject(audio) => self.extract_audio(audio),
            CreativeWorkVariant::Figure(figure) => {
                self.visit_figure(figure);
                return WalkControl::Break;
            }
            CreativeWorkVariant::ImageObject(image) => self.extract_image(image),
            CreativeWorkVariant::VideoObject(video) => self.extract_video(video),
            CreativeWorkVariant::Table(table) => {
                if let Some(images) = &mut table.options.images {
                    self.extract_images(images)
                }
            }
            _ => {}
        }
        WalkControl::Continue
    }

    fn visit_block(&mut self, block: &mut Block) -> WalkControl {
        match block {
            Block::AudioObject(audio) => self.extract_audio(audio),
            Block::CodeChunk(chunk) => {
                self.visit_code_chunk(chunk);
                return WalkControl::Break;
            }
            Block::Figure(figure) => {
                self.visit_figure(figure);
                return WalkControl::Break;
            }
            Block::ImageObject(image) => self.extract_image(image),
            Block::VideoObject(video) => self.extract_video(video),
            Block::MathBlock(math) => {
                if let Some(images) = &mut math.options.images {
                    self.extract_images(images)
                }
            }
            Block::Table(table) => {
                if let Some(images) = &mut table.options.images {
                    self.extract_images(images)
                }
            }
            _ => {}
        }
        WalkControl::Continue
    }

    fn visit_inline(&mut self, inline: &mut Inline) -> WalkControl {
        match inline {
            Inline::AudioObject(audio) => self.extract_audio(audio),
            Inline::ImageObject(image) => self.extract_image(image),
            Inline::VideoObject(video) => self.extract_video(video),
            Inline::MathInline(math) => {
                if let Some(images) = &mut math.options.images {
                    self.extract_images(images)
                }
            }
            _ => {}
        }
        WalkControl::Continue
    }
}

#[cfg(test)]
mod tests {
    use std::fs::{read, read_dir, read_to_string, write};

    use eyre::{OptionExt, Result, bail};
    use tempfile::tempdir;

    use stencila_schema::{CodeChunk, Cord, Figure, Inline, LabelType, Paragraph, Text};

    use super::*;

    const DATA_URI_1: &str = "data:image/png;base64,AA==";
    const DATA_URI_2: &str = "data:image/png;base64,AQ==";
    const SVG_DATA_URI_WITH_COMMAS: &str =
        "data:image/svg+xml,%3Csvg%3E%3Cpolygon%20points='0%200,10%200,0%2010'%2F%3E%3C%2Fsvg%3E";

    #[test]
    fn extracts_figure_media_using_figure_id() -> Result<()> {
        let media_dir = tempdir()?;
        let mut block = Block::Figure(Figure {
            id: Some("Fig 1".to_string()),
            label: Some("1".to_string()),
            caption: Some(vec![Block::Paragraph(Paragraph::new(vec![Inline::Text(
                Text::from("A figure caption. Extra detail."),
            )]))]),
            content: vec![Block::ImageObject(ImageObject::new(DATA_URI_1.to_string()))],
            ..Default::default()
        });

        let assets = extract_media_with_paths(&mut block, None, media_dir.path())?;

        let Block::Figure(figure) = block else {
            bail!("expected figure")
        };
        let Block::ImageObject(image) = figure.content.first().ok_or_eyre("image")? else {
            bail!("expected image")
        };

        assert!(image.content_url.ends_with("fig-1.png"));
        assert!(media_dir.path().join("fig-1.png").exists());
        assert_eq!(assets.len(), 1);
        assert_eq!(assets[0].node_type.as_deref(), Some("Figure"));
        assert_eq!(assets[0].role.as_deref(), Some("figure"));
        assert_eq!(
            assets[0].title.as_deref(),
            Some("Figure 1: A figure caption.")
        );
        assert_eq!(
            assets[0].description.as_deref(),
            Some("Figure 1: A figure caption. Extra detail.")
        );

        Ok(())
    }

    #[test]
    fn extracts_multiple_code_chunk_outputs_with_collision_suffix() -> Result<()> {
        let media_dir = tempdir()?;
        let mut chunk = CodeChunk::new(Cord::from("plot()"));
        chunk.id = Some("fig-1".to_string());
        chunk.outputs = Some(vec![
            Node::ImageObject(ImageObject::new(DATA_URI_1.to_string())),
            Node::ImageObject(ImageObject::new(DATA_URI_2.to_string())),
        ]);
        let mut block = Block::CodeChunk(chunk);

        extract_media(&mut block, None, media_dir.path())?;

        let Block::CodeChunk(chunk) = block else {
            bail!("expected code chunk")
        };
        let outputs = chunk.outputs.as_ref().ok_or_eyre("outputs")?;
        let Node::ImageObject(first) = &outputs[0] else {
            bail!("expected first image")
        };
        let Node::ImageObject(second) = &outputs[1] else {
            bail!("expected second image")
        };

        let second_hash = format!("{:x}", hash_bytes(DATA_URI_2.as_bytes()));
        let second_name = format!("fig-1-{second_hash}.png");

        assert!(first.content_url.ends_with("fig-1.png"));
        assert!(second.content_url.ends_with(&second_name));
        assert!(media_dir.path().join("fig-1.png").exists());
        assert!(media_dir.path().join(second_name).exists());

        Ok(())
    }

    #[test]
    fn extracts_non_base64_svg_data_uri_with_commas() -> Result<()> {
        let media_dir = tempdir()?;
        let mut block = Block::ImageObject(ImageObject::new(SVG_DATA_URI_WITH_COMMAS.to_string()));

        extract_media(&mut block, None, media_dir.path())?;

        let Block::ImageObject(image) = block else {
            bail!("expected image")
        };
        assert!(image.content_url.ends_with(".svg"));
        let extracted = media_dir.path().join(&image.content_url);
        assert_eq!(
            read_to_string(extracted)?,
            "<svg><polygon points='0 0,10 0,0 10'/></svg>"
        );

        Ok(())
    }

    #[test]
    fn extracts_subfigure_code_chunk_output_using_existing_id() -> Result<()> {
        let media_dir = tempdir()?;
        let mut chunk = CodeChunk::new(Cord::from("plot()"));
        chunk.id = Some("fig-1a".to_string());
        chunk.label_type = Some(LabelType::FigureLabel);
        chunk.label = Some("1a".to_string());
        chunk.caption = Some(vec![Block::Paragraph(Paragraph::new(vec![Inline::Text(
            Text::from("A plot. Extra detail."),
        )]))]);
        chunk.outputs = Some(vec![Node::ImageObject(ImageObject::new(
            DATA_URI_1.to_string(),
        ))]);
        let mut block = Block::Figure(Figure {
            id: Some("fig-1".to_string()),
            content: vec![Block::CodeChunk(chunk)],
            ..Default::default()
        });

        let assets = extract_media_with_paths(&mut block, None, media_dir.path())?;

        let Block::Figure(figure) = block else {
            bail!("expected figure")
        };
        let Block::CodeChunk(chunk) = figure.content.first().ok_or_eyre("chunk")? else {
            bail!("expected code chunk")
        };
        let outputs = chunk.outputs.as_ref().ok_or_eyre("outputs")?;
        let Node::ImageObject(first) = &outputs[0] else {
            bail!("expected image")
        };

        assert!(first.content_url.ends_with("fig-1a.png"));
        assert!(media_dir.path().join("fig-1a.png").exists());
        assert_eq!(assets.len(), 1);
        assert_eq!(assets[0].node_type.as_deref(), Some("CodeChunk"));
        assert_eq!(assets[0].role.as_deref(), Some("computational-output"));
        assert_eq!(assets[0].title.as_deref(), Some("Figure 1a: A plot."));
        assert_eq!(
            assets[0].description.as_deref(),
            Some("Figure 1a: A plot. Extra detail.")
        );

        Ok(())
    }

    #[test]
    fn extracts_caption_media_without_consuming_figure_stem() -> Result<()> {
        let media_dir = tempdir()?;
        let mut block = Block::Figure(Figure {
            id: Some("fig-1".to_string()),
            caption: Some(vec![Block::ImageObject(ImageObject::new(
                DATA_URI_1.to_string(),
            ))]),
            content: vec![Block::ImageObject(ImageObject::new(DATA_URI_2.to_string()))],
            ..Default::default()
        });

        extract_media(&mut block, None, media_dir.path())?;

        let Block::Figure(figure) = block else {
            bail!("expected figure")
        };
        let Block::ImageObject(caption_image) = figure
            .caption
            .as_ref()
            .and_then(|caption| caption.first())
            .ok_or_eyre("caption image")?
        else {
            bail!("expected caption image")
        };
        let Block::ImageObject(content_image) =
            figure.content.first().ok_or_eyre("content image")?
        else {
            bail!("expected content image")
        };

        assert!(!caption_image.content_url.ends_with("fig-1.png"));
        assert!(content_image.content_url.ends_with("fig-1.png"));
        assert!(media_dir.path().join("fig-1.png").exists());

        Ok(())
    }

    #[test]
    fn extracts_identical_media_once_across_readable_stems() -> Result<()> {
        let media_dir = tempdir()?;
        let mut blocks = vec![
            Block::Figure(Figure {
                id: Some("fig-one".to_string()),
                content: vec![Block::ImageObject(ImageObject::new(DATA_URI_1.to_string()))],
                ..Default::default()
            }),
            Block::Figure(Figure {
                id: Some("fig-two".to_string()),
                content: vec![Block::ImageObject(ImageObject::new(DATA_URI_1.to_string()))],
                ..Default::default()
            }),
        ];

        extract_media(&mut blocks, None, media_dir.path())?;

        let Block::Figure(first_figure) = &blocks[0] else {
            bail!("expected first figure")
        };
        let Block::Figure(second_figure) = &blocks[1] else {
            bail!("expected second figure")
        };
        let Block::ImageObject(first_image) =
            first_figure.content.first().ok_or_eyre("first image")?
        else {
            bail!("expected first image")
        };
        let Block::ImageObject(second_image) =
            second_figure.content.first().ok_or_eyre("second image")?
        else {
            bail!("expected second image")
        };

        assert_eq!(first_image.content_url, second_image.content_url);
        assert!(first_image.content_url.ends_with("fig-one.png"));
        assert_eq!(read_dir(media_dir.path())?.count(), 1);

        Ok(())
    }

    #[test]
    fn extracts_over_existing_mutated_media_in_place() -> Result<()> {
        let media_dir = tempdir()?;
        let mut first = Block::Figure(Figure {
            id: Some("fig-1".to_string()),
            content: vec![Block::ImageObject(ImageObject::new(DATA_URI_1.to_string()))],
            ..Default::default()
        });

        extract_media(&mut first, None, media_dir.path())?;
        let media_path = media_dir.path().join("fig-1.png");
        write(&media_path, b"previously signed bytes")?;

        let mut second = Block::Figure(Figure {
            id: Some("fig-1".to_string()),
            content: vec![Block::ImageObject(ImageObject::new(DATA_URI_1.to_string()))],
            ..Default::default()
        });
        extract_media(&mut second, None, media_dir.path())?;

        let Block::Figure(figure) = second else {
            bail!("expected figure")
        };
        let Block::ImageObject(image) = figure.content.first().ok_or_eyre("image")? else {
            bail!("expected image")
        };

        assert!(image.content_url.ends_with("fig-1.png"));
        assert_eq!(read(&media_path)?, [0u8]);
        assert_eq!(read_dir(media_dir.path())?.count(), 1);

        Ok(())
    }
}
