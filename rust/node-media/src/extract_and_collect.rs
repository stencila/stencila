use std::{
    env::current_dir,
    fs::{File, create_dir_all},
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
};

use eyre::Result;
use seahash::SeaHasher;

use stencila_schema::{
    AudioObject, Block, CreativeWorkVariant, ImageObject, Inline, Node, VideoObject, VisitorMut,
    WalkControl, WalkNode,
};

use crate::extract_media;

/// Information about a media file that has been extracted or collected
#[derive(Debug, Clone)]
pub struct MediaFile {
    /// Path to the file in the temporary directory
    pub path: PathBuf,

    /// Original content_url from the node
    pub content_url: String,

    /// Content hash (hexadecimal)
    pub hash: String,

    /// File extension (without dot)
    pub extension: String,
}

/// Extract data URIs and collect all media files
///
/// This function:
///
/// 1. Extracts data URIs to files (using `extract_media`)
/// 2. Collects all media files including those with relative paths
/// 3. Copies relative path files to the media directory with content-based hashes
/// 4. Returns a list of all media files extracted or collected
///
/// # Arguments
/// * `node` - The document node tree (will be mutated with updated content_urls)
/// * `document_path` - Path to the source document (for resolving relative paths)
/// * `media_dir` - Directory where media files will be extracted/collected
///
/// # Returns
/// A vector of [`MediaFile`] structs containing information about each media file
pub fn extract_and_collect_media<T>(
    node: &mut T,
    document_path: Option<&Path>,
    media_dir: &Path,
) -> Result<Vec<MediaFile>>
where
    T: WalkNode,
{
    // First, extract any data URIs to files
    extract_media(node, document_path, media_dir)?;

    // Determine the document directory for resolving relative paths
    let document_dir = match document_path {
        Some(path) => {
            // Get the parent directory of the document
            match path.parent() {
                // If parent exists and is not empty, use it
                Some(parent) if !parent.as_os_str().is_empty() => parent,
                // If parent is empty or None, the document is at the current directory
                _ => Path::new("."),
            }
        }
        None => &current_dir()?,
    };

    // Now collect all media files (including those that were just extracted
    // and those that have relative/absolute file paths)
    let mut collector = Collector {
        document_dir: document_dir.into(),
        media_dir: media_dir.into(),
        media_files: Vec::new(),
    };
    collector.walk(node);

    Ok(collector.media_files)
}

struct Collector {
    /// The directory containing the document (for resolving relative paths)
    document_dir: PathBuf,

    /// The directory where media files are stored
    media_dir: PathBuf,

    /// Collected media files
    media_files: Vec<MediaFile>,
}

impl Collector {
    /// Process an image and collect it if it's a file path
    fn collect_image(&mut self, image: &ImageObject) {
        self.collect_media(&image.content_url, "image");
    }

    /// Process an audio file and collect it if it's a file path
    fn collect_audio(&mut self, audio: &AudioObject) {
        self.collect_media(&audio.content_url, "audio");
    }

    /// Process a video file and collect it if it's a file path
    fn collect_video(&mut self, video: &VideoObject) {
        self.collect_media(&video.content_url, "video");
    }

    /// Collect media from a content URL
    fn collect_media(&mut self, content_url: &str, media_type: &str) {
        // Skip data URIs (already extracted) and external URLs
        if content_url.starts_with("data:")
            || content_url.starts_with("http://")
            || content_url.starts_with("https://")
        {
            return;
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
            return;
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
        let (hash_str, dest_path) = match self.hash_and_copy_file(&source_path, &extension) {
            Ok(result) => result,
            Err(error) => {
                tracing::error!(
                    "Failed to process media file {}: {error}",
                    source_path.display()
                );
                return;
            }
        };

        // Add to collected media files
        self.media_files.push(MediaFile {
            path: dest_path,
            content_url: content_url.to_string(),
            hash: hash_str,
            extension,
        });
    }

    /// Hash and copy a file in a single pass to avoid reading twice
    ///
    /// Returns the hash string and destination path.
    /// Creates the destination file only if it doesn't already exist.
    fn hash_and_copy_file(&self, source_path: &Path, extension: &str) -> Result<(String, PathBuf)> {
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

        Ok((hash_str, dest_path))
    }

    fn collect_images(&mut self, images: &[ImageObject]) {
        images.iter().for_each(|image| self.collect_image(image));
    }
}

impl VisitorMut for Collector {
    fn visit_node(&mut self, node: &mut Node) -> WalkControl {
        match node {
            Node::AudioObject(audio) => self.collect_audio(audio),
            Node::ImageObject(image) => self.collect_image(image),
            Node::VideoObject(video) => self.collect_video(video),
            Node::MathBlock(math) => {
                if let Some(images) = &math.options.images {
                    self.collect_images(images)
                }
            }
            Node::MathInline(math) => {
                if let Some(images) = &math.options.images {
                    self.collect_images(images)
                }
            }
            Node::Table(table) => {
                if let Some(images) = &table.options.images {
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
                if let Some(images) = &table.options.images {
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
                if let Some(images) = &math.options.images {
                    self.collect_images(images)
                }
            }
            Block::Table(table) => {
                if let Some(images) = &table.options.images {
                    self.collect_images(images)
                }
            }
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
                if let Some(images) = &math.options.images {
                    self.collect_images(images)
                }
            }
            _ => {}
        }
        WalkControl::Continue
    }
}
