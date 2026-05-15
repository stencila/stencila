use std::{
    env::current_dir,
    fs::File,
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
    sync::LazyLock,
};

use eyre::Result;
use pathdiff::diff_paths;
use regex::Regex;
use seahash::SeaHasher;

use stencila_codec_info::EncodedAsset;
use stencila_schema::{
    AudioObject, Block, CodeChunk, Cord, CreativeWorkVariant, Figure, ImageObject, Inline, Node,
    NodeId, NodeType, RawBlock, VideoObject, VisitorMut, WalkControl, WalkNode,
};

use crate::naming::MediaNamer;

/// Collect all media files
///
/// # Arguments
/// * `node` - The document node tree (will be mutated with updated content_urls)
/// * `document_path` - Path to the source document (for resolving relative paths)
/// * `to_path` - Path of the destination file (for creating relative paths to collected files)
/// * `media_dir` - Directory where media files will be extracted/collected
pub fn collect_media<T>(
    node: &mut T,
    document_path: Option<&Path>,
    to_path: &Path,
    media_dir: &Path,
) -> Result<()>
where
    T: WalkNode,
{
    collect_media_with_paths(node, document_path, to_path, media_dir).map(|_| ())
}

/// Collect media files and return a record per asset copied or referenced.
///
/// Each [`EncodedAsset`] is annotated with the originating node's id/type and
/// an asset role so dispatchers can attach per-node provenance to the file.
pub fn collect_media_with_paths<T>(
    node: &mut T,
    document_path: Option<&Path>,
    to_path: &Path,
    media_dir: &Path,
) -> Result<Vec<EncodedAsset>>
where
    T: WalkNode,
{
    // Determine the document directory for resolving relative paths
    let document_dir = match document_path {
        Some(path) => {
            // Get the parent directory of the source document
            match path.parent() {
                // If parent exists and is not empty, use it
                Some(parent) if !parent.as_os_str().is_empty() => parent,
                // If parent is empty or None, the document is at the current directory
                _ => Path::new("."),
            }
        }
        None => &current_dir()?,
    };

    // Determine the destination directory for calculating relative paths
    let to_dir = match to_path.parent() {
        // If parent exists and is not empty, use it
        Some(parent) if !parent.as_os_str().is_empty() => parent,
        // If parent is empty or None, the document is at the current directory
        _ => Path::new("."),
    };

    let mut collector = Collector {
        document_dir: document_dir.into(),
        to_dir: to_dir.into(),
        media_dir: media_dir.into(),
        parent_stack: Vec::new(),
        namer: MediaNamer::with_hashed_readable_names(),
        assets: Vec::new(),
    };
    collector.walk(node);

    Ok(collector.assets)
}

struct Collector {
    /// The directory containing the document (for resolving relative paths)
    document_dir: PathBuf,

    /// The directory containing the destination file
    to_dir: PathBuf,

    /// The directory where media files are stored
    media_dir: PathBuf,

    /// Stack of ancestor structs, used to attribute collected assets to the
    /// closest meaningful originating node.
    parent_stack: Vec<(NodeType, NodeId)>,

    /// State used to derive readable media filenames from nearby node ids.
    namer: MediaNamer,

    /// The asset records produced during collection.
    assets: Vec<EncodedAsset>,
}

impl Collector {
    /// Collect media from a content URL
    ///
    /// Returns the relative URL to use for the collected media file paired
    /// with the absolute path of the file written, or `None` if the URL
    /// doesn't need to be rewritten.
    fn collect_media(
        &mut self,
        content_url: &str,
        media_type: &str,
        desired_stem: Option<&str>,
    ) -> Option<(String, PathBuf)> {
        // Skip data URIs and external URLs
        if !should_collect_url(content_url) {
            return None;
        }

        // This is a file path - resolve it
        let path = Path::new(content_url);
        let source_path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.document_dir.join(path)
        };

        // Check if the file exists
        if !source_path.exists() {
            tracing::warn!("Media file not found: {}", source_path.display());
            return None;
        }

        // Determine extension first (before opening file)
        let extension = source_path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or(match media_type {
                "image" => "png",
                "audio" => "mp3",
                "video" => "mp4",
                _ => "bin",
            })
            .to_string();

        // Hash and copy the file in a single pass to avoid reading twice
        let dest_path = match self.hash_and_copy_file(&source_path, &extension, desired_stem) {
            Ok(result) => result,
            Err(error) => {
                tracing::error!(
                    "Failed to process media file {}: {error}",
                    source_path.display()
                );
                return None;
            }
        };

        // Create relative URL from document directory to the media file
        let relative_url = diff_paths(&dest_path, &self.to_dir)
            .unwrap_or_else(|| dest_path.clone())
            .to_string_lossy()
            .to_string();

        Some((relative_url, dest_path))
    }

    /// Record a collected asset with originating-node attribution.
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

    /// Compute the originating node for an asset (executable preferred,
    /// then math/table containers, then the media object itself).
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

    /// Hash and copy a file in a single pass to avoid reading twice
    ///
    /// Returns the hash string and destination path.
    /// Creates the destination file only if it doesn't already exist.
    fn hash_and_copy_file(
        &mut self,
        source_path: &Path,
        extension: &str,
        desired_stem: Option<&str>,
    ) -> Result<PathBuf> {
        use std::io::{BufReader, Read};

        // Open source file for reading
        let source_file = File::open(source_path)?;
        let mut reader = BufReader::new(source_file);

        // Initialize hasher
        let mut hasher = SeaHasher::new();
        let mut buffer = [0u8; 8192]; // 8KB buffer for streaming

        // First pass: compute hash by streaming through the file
        loop {
            let bytes_read = reader.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            buffer[..bytes_read].hash(&mut hasher);
        }

        let hash = hasher.finish();
        self.namer
            .copy_file(source_path, &self.media_dir, desired_stem, extension, hash)
    }

    /// Collect an image
    fn collect_image(&mut self, image: &mut ImageObject) {
        if image.is_viz() || !should_collect_url(&image.content_url) {
            return;
        }

        let desired_stem = self.namer.next_media_stem(image.id.as_deref());
        let title = self.namer.next_media_title(image.title.as_deref());
        let description = self.namer.next_media_description(image.title.as_deref());
        if let Some((relative_url, path)) =
            self.collect_media(&image.content_url, "image", desired_stem.as_deref())
        {
            let id = image.node_id();
            self.record_asset(path, Some(&id), NodeType::ImageObject, title, description);
            image.content_url = relative_url;
        }
    }

    /// Collect one or more images
    fn collect_images(&mut self, images: &mut [ImageObject]) {
        images
            .iter_mut()
            .for_each(|image| self.collect_image(image));
    }

    /// Collect an audio file
    fn collect_audio(&mut self, audio: &mut AudioObject) {
        if !should_collect_url(&audio.content_url) {
            return;
        }

        let desired_stem = self.namer.next_media_stem(audio.id.as_deref());
        let title = self.namer.next_media_title(audio.title.as_deref());
        let description = self.namer.next_media_description(audio.title.as_deref());
        if let Some((relative_url, path)) =
            self.collect_media(&audio.content_url, "audio", desired_stem.as_deref())
        {
            let id = audio.node_id();
            self.record_asset(path, Some(&id), NodeType::AudioObject, title, description);
            audio.content_url = relative_url;
        }
    }

    /// Collect a video file
    fn collect_video(&mut self, video: &mut VideoObject) {
        if !should_collect_url(&video.content_url) {
            return;
        }

        let desired_stem = self.namer.next_media_stem(video.id.as_deref());
        let title = self.namer.next_media_title(video.title.as_deref());
        let description = self.namer.next_media_description(video.title.as_deref());
        if let Some((relative_url, path)) =
            self.collect_media(&video.content_url, "video", desired_stem.as_deref())
        {
            let id = video.node_id();
            self.record_asset(path, Some(&id), NodeType::VideoObject, title, description);
            video.content_url = relative_url;
        }
    }

    fn collect_figure(&mut self, figure: &mut Figure) {
        self.parent_stack.push((NodeType::Figure, figure.node_id()));

        if let Some(caption) = &mut figure.caption {
            caption.walk_mut(self);
        }

        self.namer.push_figure(figure);
        figure.content.walk_mut(self);
        self.namer.pop();

        self.parent_stack.pop();
    }

    fn collect_code_chunk(&mut self, chunk: &mut CodeChunk) {
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

    /// Collect media from HTML content in RawBlock nodes
    fn collect_html_media(&mut self, raw_block: &mut RawBlock) {
        // Only process HTML format
        if raw_block.format.to_lowercase() != "html" {
            return;
        }

        let html = raw_block.content.to_string();
        let mut modified_html = html.clone();

        // Regex pattern to match img, video, and audio tags with src attributes
        // This pattern captures the entire tag and the src attribute value
        static MEDIA_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
            Regex::new(r#"<(img|video|audio)([^>]*?)\s+src\s*=\s*["']([^"']+)["']([^>]*?)(/?)>"#)
                .expect("invalid regex")
        });

        // Find all media tags and collect their sources
        let matches: Vec<_> = MEDIA_PATTERN.captures_iter(&html).collect();

        for cap in matches {
            let full_match = &cap[0];
            let tag_name = &cap[1];
            let before_src = &cap[2];
            let src_value = &cap[3];
            let after_src = &cap[4];
            let self_closing = &cap[5];

            // Determine media type from tag name and node type for attribution
            let (media_type, node_type) = match tag_name {
                "img" => ("image", NodeType::ImageObject),
                "video" => ("video", NodeType::VideoObject),
                "audio" => ("audio", NodeType::AudioObject),
                _ => continue,
            };

            // Attempt to collect the media file
            if !should_collect_url(src_value) {
                continue;
            }

            let desired_stem = self.namer.next_media_stem(None);
            if let Some((new_url, path)) =
                self.collect_media(src_value, media_type, desired_stem.as_deref())
            {
                self.record_asset(path, None, node_type, None, None);

                // Reconstruct the tag with the new URL
                let new_tag =
                    format!(r#"<{tag_name}{before_src} src="{new_url}"{after_src}{self_closing}>"#);

                // Replace the old tag with the new one
                modified_html = modified_html.replace(full_match, &new_tag);
            }
        }

        // Update the raw block content if any changes were made
        if modified_html != html {
            raw_block.content = Cord::from(modified_html);
        }
    }
}

/// See [`extract::is_executable`]; collected media follow the same attribution
/// rules so dispatchers can treat extracted and collected assets uniformly.
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

fn is_media_container(node_type: NodeType) -> bool {
    matches!(
        node_type,
        NodeType::MathBlock | NodeType::MathInline | NodeType::Table | NodeType::Figure
    )
}

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

fn should_collect_url(content_url: &str) -> bool {
    !content_url.starts_with("data:")
        && !content_url.starts_with("http://")
        && !content_url.starts_with("https://")
}

impl VisitorMut for Collector {
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
                self.collect_code_chunk(chunk);
                return WalkControl::Break;
            }
            Node::Figure(figure) => {
                self.collect_figure(figure);
                return WalkControl::Break;
            }
            Node::AudioObject(audio) => self.collect_audio(audio),
            Node::ImageObject(image) => self.collect_image(image),
            Node::VideoObject(video) => self.collect_video(video),
            Node::MathBlock(math) => {
                if let Some(images) = &mut math.options.images {
                    self.collect_images(images)
                }
            }
            Node::MathInline(math) => {
                if let Some(images) = &mut math.options.images {
                    self.collect_images(images)
                }
            }
            Node::Table(table) => {
                if let Some(images) = &mut table.options.images {
                    self.collect_images(images)
                }
            }
            _ => {}
        }
        WalkControl::Continue
    }

    fn visit_work(&mut self, work: &mut CreativeWorkVariant) -> WalkControl {
        match work {
            CreativeWorkVariant::AudioObject(audio) => self.collect_audio(audio),
            CreativeWorkVariant::Figure(figure) => {
                self.collect_figure(figure);
                return WalkControl::Break;
            }
            CreativeWorkVariant::ImageObject(image) => self.collect_image(image),
            CreativeWorkVariant::VideoObject(video) => self.collect_video(video),
            CreativeWorkVariant::Table(table) => {
                if let Some(images) = &mut table.options.images {
                    self.collect_images(images)
                }
            }
            _ => {}
        }
        WalkControl::Continue
    }

    fn visit_block(&mut self, block: &mut Block) -> WalkControl {
        match block {
            Block::AudioObject(audio) => self.collect_audio(audio),
            Block::CodeChunk(chunk) => {
                self.collect_code_chunk(chunk);
                return WalkControl::Break;
            }
            Block::Figure(figure) => {
                self.collect_figure(figure);
                return WalkControl::Break;
            }
            Block::ImageObject(image) => self.collect_image(image),
            Block::VideoObject(video) => self.collect_video(video),
            Block::MathBlock(math) => {
                if let Some(images) = &mut math.options.images {
                    self.collect_images(images)
                }
            }
            Block::Table(table) => {
                if let Some(images) = &mut table.options.images {
                    self.collect_images(images)
                }
            }
            Block::RawBlock(raw_block) => self.collect_html_media(raw_block),
            _ => {}
        }
        WalkControl::Continue
    }

    fn visit_inline(&mut self, inline: &mut Inline) -> WalkControl {
        match inline {
            Inline::AudioObject(audio) => self.collect_audio(audio),
            Inline::ImageObject(image) => self.collect_image(image),
            Inline::VideoObject(video) => self.collect_video(video),
            Inline::MathInline(math) => {
                if let Some(images) = &mut math.options.images {
                    self.collect_images(images)
                }
            }
            _ => {}
        }
        WalkControl::Continue
    }
}

#[cfg(test)]
mod tests {
    use std::fs::{read, read_dir, write};

    use eyre::{OptionExt, Result, bail};
    use tempfile::tempdir;

    use stencila_schema::{CodeChunk, Cord, Figure, Inline, LabelType, Paragraph, Text};

    use crate::naming::hash_bytes;

    use super::*;

    #[test]
    fn collects_subfigure_code_chunk_outputs_using_existing_id() -> Result<()> {
        let dir = tempdir()?;
        let document_path = dir.path().join("source.md");
        let output_path = dir.path().join("public").join("index.html");
        let media_dir = dir.path().join("public").join("media");
        write(&document_path, "")?;
        write(dir.path().join("plot-one.png"), [0u8])?;
        write(dir.path().join("plot-two.png"), [1u8])?;

        let mut chunk = CodeChunk::new(Cord::from("plot()"));
        chunk.id = Some("fig-1a".to_string());
        chunk.label_type = Some(LabelType::FigureLabel);
        chunk.label = Some("1a".to_string());
        chunk.caption = Some(vec![Block::Paragraph(Paragraph::new(vec![Inline::Text(
            Text::from("A plot. Extra detail."),
        )]))]);
        chunk.outputs = Some(vec![
            Node::ImageObject(ImageObject::new("plot-one.png".to_string())),
            Node::ImageObject(ImageObject::new("plot-two.png".to_string())),
        ]);
        let mut block = Block::Figure(Figure {
            id: Some("fig-1".to_string()),
            content: vec![Block::CodeChunk(chunk)],
            ..Default::default()
        });

        let assets =
            collect_media_with_paths(&mut block, Some(&document_path), &output_path, &media_dir)?;

        let Block::Figure(figure) = block else {
            bail!("expected figure")
        };
        let Block::CodeChunk(chunk) = figure.content.first().ok_or_eyre("chunk")? else {
            bail!("expected code chunk")
        };
        let outputs = chunk.outputs.as_ref().ok_or_eyre("outputs")?;
        let Node::ImageObject(first) = &outputs[0] else {
            bail!("expected first image")
        };
        let Node::ImageObject(second) = &outputs[1] else {
            bail!("expected second image")
        };

        let first_hash = format!("{:x}", hash_bytes(&[0u8]));
        let second_hash = format!("{:x}", hash_bytes(&[1u8]));
        let first_name = format!("fig-1a-{first_hash}.png");
        let second_name = format!("fig-1a-{second_hash}.png");

        assert_eq!(first.content_url, format!("media/{first_name}"));
        assert_eq!(second.content_url, format!("media/{second_name}"));
        assert!(media_dir.join(first_name).exists());
        assert!(media_dir.join(second_name).exists());
        assert_eq!(assets.len(), 2);
        assert!(
            assets
                .iter()
                .all(|asset| asset.node_type.as_deref() == Some("CodeChunk"))
        );
        assert!(
            assets
                .iter()
                .all(|asset| asset.role.as_deref() == Some("computational-output"))
        );
        assert!(
            assets
                .iter()
                .all(|asset| asset.title.as_deref() == Some("Figure 1a: A plot."))
        );
        assert!(
            assets
                .iter()
                .all(|asset| asset.description.as_deref()
                    == Some("Figure 1a: A plot. Extra detail."))
        );

        Ok(())
    }

    #[test]
    fn collects_identical_media_once_across_readable_stems() -> Result<()> {
        let dir = tempdir()?;
        let document_path = dir.path().join("source.md");
        let output_path = dir.path().join("public").join("index.html");
        let media_dir = dir.path().join("public").join("media");
        write(&document_path, "")?;
        write(dir.path().join("shared.png"), [0u8])?;

        let mut first = ImageObject::new("shared.png".to_string());
        first.id = Some("img-one".to_string());
        let mut second = ImageObject::new("shared.png".to_string());
        second.id = Some("img-two".to_string());
        let mut blocks = vec![Block::ImageObject(first), Block::ImageObject(second)];

        collect_media(&mut blocks, Some(&document_path), &output_path, &media_dir)?;

        let Block::ImageObject(first) = &blocks[0] else {
            bail!("expected first image")
        };
        let Block::ImageObject(second) = &blocks[1] else {
            bail!("expected second image")
        };

        let hash = format!("{:x}", hash_bytes(&[0u8]));
        assert_eq!(first.content_url, format!("media/img-one-{hash}.png"));
        assert_eq!(second.content_url, first.content_url);
        assert_eq!(read_dir(&media_dir)?.count(), 1);

        Ok(())
    }

    #[test]
    fn collects_over_existing_mutated_media_in_place() -> Result<()> {
        let dir = tempdir()?;
        let document_path = dir.path().join("source.md");
        let output_path = dir.path().join("public").join("index.html");
        let media_dir = dir.path().join("public").join("media");
        write(&document_path, "")?;
        write(dir.path().join("plot.png"), [0u8])?;

        let mut first = ImageObject::new("plot.png".to_string());
        first.id = Some("fig-1".to_string());
        let mut first = Block::ImageObject(first);
        collect_media(&mut first, Some(&document_path), &output_path, &media_dir)?;

        let hash = format!("{:x}", hash_bytes(&[0u8]));
        let media_path = media_dir.join(format!("fig-1-{hash}.png"));
        write(&media_path, b"previously signed bytes")?;

        let mut second = ImageObject::new("plot.png".to_string());
        second.id = Some("fig-1".to_string());
        let mut second = Block::ImageObject(second);
        collect_media(&mut second, Some(&document_path), &output_path, &media_dir)?;

        let Block::ImageObject(image) = second else {
            bail!("expected image")
        };

        assert_eq!(image.content_url, format!("media/fig-1-{hash}.png"));
        assert_eq!(read(&media_path)?, [0u8]);
        assert_eq!(read_dir(&media_dir)?.count(), 1);

        Ok(())
    }
}
