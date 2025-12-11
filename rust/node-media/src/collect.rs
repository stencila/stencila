use std::{
    env::current_dir,
    fs::{File, create_dir_all},
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
    sync::LazyLock,
};

use eyre::Result;
use pathdiff::diff_paths;
use regex::Regex;
use seahash::SeaHasher;

use stencila_schema::{
    AudioObject, Block, Cord, CreativeWorkVariant, ImageObject, Inline, Node, RawBlock,
    VideoObject, VisitorMut, WalkControl, WalkNode,
};

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
    };
    collector.walk(node);

    Ok(())
}

struct Collector {
    /// The directory containing the document (for resolving relative paths)
    document_dir: PathBuf,

    /// The directory containing the destination file
    to_dir: PathBuf,

    /// The directory where media files are stored
    media_dir: PathBuf,
}

impl Collector {
    /// Collect media from a content URL
    ///
    /// Returns the relative URL to use for the collected media file,
    /// or None if the URL doesn't need to be rewritten.
    fn collect_media(&mut self, content_url: &str, media_type: &str) -> Option<String> {
        // Skip data URIs and external URLs
        if content_url.starts_with("data:")
            || content_url.starts_with("http://")
            || content_url.starts_with("https://")
        {
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
        let dest_path = match self.hash_and_copy_file(&source_path, &extension) {
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

        Some(relative_url)
    }

    /// Hash and copy a file in a single pass to avoid reading twice
    ///
    /// Returns the hash string and destination path.
    /// Creates the destination file only if it doesn't already exist.
    fn hash_and_copy_file(&self, source_path: &Path, extension: &str) -> Result<PathBuf> {
        use std::io::{BufReader, BufWriter, Read, Write};

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
        let hash_str = format!("{hash:x}");

        // Create destination path
        let media_name = format!("{hash_str}.{extension}");
        let dest_path = self.media_dir.join(&media_name);

        // Copy the file if it doesn't already exist
        if !dest_path.exists() {
            create_dir_all(&self.media_dir)?;

            // Reopen source file for copying
            let source_file = File::open(source_path)?;
            let mut reader = BufReader::new(source_file);

            let dest_file = File::create(&dest_path)?;
            let mut writer = BufWriter::new(dest_file);

            // Copy with buffering
            std::io::copy(&mut reader, &mut writer)?;
            writer.flush()?;
        }

        Ok(dest_path)
    }

    /// Collect an image
    fn collect_image(&mut self, image: &mut ImageObject) {
        if !image.is_viz()
            && let Some(relative_url) = self.collect_media(&image.content_url, "image")
        {
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
        if let Some(relative_url) = self.collect_media(&audio.content_url, "audio") {
            audio.content_url = relative_url;
        }
    }

    /// Collect a video file
    fn collect_video(&mut self, video: &mut VideoObject) {
        if let Some(relative_url) = self.collect_media(&video.content_url, "video") {
            video.content_url = relative_url;
        }
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

            // Determine media type from tag name
            let media_type = match tag_name {
                "img" => "image",
                "video" => "video",
                "audio" => "audio",
                _ => continue,
            };

            // Attempt to collect the media file
            if let Some(new_url) = self.collect_media(src_value, media_type) {
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

impl VisitorMut for Collector {
    fn visit_node(&mut self, node: &mut Node) -> WalkControl {
        match node {
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
