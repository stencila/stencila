use std::{
    env::current_dir,
    io::Cursor,
    path::{Path, PathBuf},
};

use base64::{Engine as _, engine::general_purpose::STANDARD};
use eyre::{Context, Result, bail, eyre};
use image::{GenericImageView, ImageFormat, ImageReader};
use tempfile::NamedTempFile;

use stencila_format::Format;
use stencila_images::ImageResizeOptions;
use stencila_schema::{
    AudioObject, Block, CreativeWorkVariant, ImageObject, Inline, Node, VideoObject, VisitorMut,
    WalkControl, WalkNode,
};
use stencila_tools::{Convert, Ffmpeg, Tool};

/// Embed media files within [`ImageObject`], [`VideoObject`], and
/// [`AudioObject`] as dataURIs
///
/// Videos are optimized using FFmpeg for web viewing by scaling to 720p,
/// using 24fps framerate, and balanced quality settings. Audio files are
/// optimized by converting to MP3 with 128k bitrate for good web quality.
///
/// For each media object, common file extensions are tried because sometimes
/// the object's `content_url` does not include a file extension. The empty
/// extension ("") is tried first in case the `content_url` already includes the
/// extension.
///
/// Extensions tried:
/// - Images: .png, .jpg, .jpeg, .gif, .tif, .tiff
/// - Videos: .mp4, .avi, .mov, .mkv, .webm, .wmv
/// - Audio: .mp3, .wav, .flac, .ogg, .aac, .m4a
///
/// This function does not return errors for individual media processing failures.
/// Instead, failures are logged as warnings or errors, allowing the embedding
/// process to continue for other media objects in the document.
///
/// Uses default web viewing options (1200px max width).
pub fn embed_media<T>(node: &mut T, path: Option<&Path>) -> Result<()>
where
    T: WalkNode,
{
    embed_media_with(node, path, ImageResizeOptions::for_web())
}

/// Embed media files with custom image options
///
/// Like [`embed_media`] but allows specifying custom [`ImageResizeOptions`] for
/// image resizing and optimization.
///
/// # Example
/// ```ignore
/// use stencila_images::ImageResizeOptions;
/// use stencila_node_media::embed_media_with;
///
/// // Use email-optimized settings (600px max width)
/// embed_media_with(&mut node, Some(path), ImageResizeOptions::for_email())?;
/// ```
pub fn embed_media_with<T>(
    node: &mut T,
    path: Option<&Path>,
    image_options: ImageResizeOptions,
) -> Result<()>
where
    T: WalkNode,
{
    let mut embedder = Embedder::new(path, None, image_options)?;
    embedder.walk(node);

    Ok(())
}

/// Embed an in individual image object
///
/// Use this when you want to embed an individual image, rather than all media
/// nested within some other node.
///
/// Uses default web viewing options (1200px max width).
pub fn embed_image(
    image: &mut ImageObject,
    path: Option<&Path>,
    format: Option<Format>,
) -> Result<()> {
    embed_image_with(image, path, format, ImageResizeOptions::for_web())
}

/// Embed an individual image object with custom options
///
/// Like [`embed_image`] but allows specifying custom [`ImageResizeOptions`] for
/// image resizing and optimization.
pub fn embed_image_with(
    image: &mut ImageObject,
    path: Option<&Path>,
    format: Option<Format>,
    image_options: ImageResizeOptions,
) -> Result<()> {
    let embedder = Embedder::new(path, format, image_options)?;
    embedder.embed_image(image);

    Ok(())
}

/// Embed an in individual audio object
///
/// Use this when you want to embed an individual audio, rather than all media
/// nested within some other node.
pub fn embed_audio(audio: &mut AudioObject, path: Option<&Path>) -> Result<()> {
    let embedder = Embedder::new(path, None, ImageResizeOptions::default())?;
    embedder.embed_audio(audio);

    Ok(())
}

/// Embed an in individual video object
///
/// Use this when you want to embed an individual video, rather than all media
/// nested within some other node.
pub fn embed_video(video: &mut VideoObject, path: Option<&Path>) -> Result<()> {
    let embedder = Embedder::new(path, None, ImageResizeOptions::default())?;
    embedder.embed_video(video);

    Ok(())
}

struct Embedder {
    /// The base directory for relative filesystem paths
    dir: PathBuf,

    /// The desired format for embedded images
    ///
    /// If the image is not already in this format it will be converted to it.
    image_format: Option<Format>,

    /// Options for image processing (resizing, etc.)
    image_options: ImageResizeOptions,
}

impl Embedder {
    fn new(
        path: Option<&Path>,
        image_format: Option<Format>,
        image_options: ImageResizeOptions,
    ) -> Result<Self> {
        let dir = if let Some(path) = path {
            let path = path
                .canonicalize()
                .wrap_err_with(|| eyre!("Path does not exist `{}`", path.display()))?;

            let dir = if path.is_file()
                && let Some(parent) = path.parent()
            {
                parent
            } else {
                &path
            };

            if !dir.exists() {
                bail!("Directory does not exist: {}", dir.display());
            }

            dir.to_path_buf()
        } else {
            current_dir()?
        };

        Ok(Self {
            dir,
            image_format,
            image_options,
        })
    }

    /// Resolve a media file path, handling both absolute and relative paths
    fn resolve_path(&self, content_url: &str, extension: &str) -> PathBuf {
        let path_with_ext = [content_url, extension].concat();
        let path = Path::new(&path_with_ext);

        if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.dir.join(path)
        }
    }

    /// Embed a vector of images
    fn embed_images(&self, images: &mut [ImageObject]) {
        images.iter_mut().for_each(|image| self.embed_image(image));
    }

    /// Embed an image by converting to a data URI using optimized settings
    fn embed_image(&self, image: &mut ImageObject) {
        if image.content_url.starts_with("data:") || image.content_url.starts_with("http") {
            return;
        }

        const IMAGE_EXTENSIONS: &[&str] =
            &["", ".png", ".jpg", ".jpeg", ".gif", ".tif", ".tiff", ".pdf"];

        for ext in IMAGE_EXTENSIONS {
            let path = self.resolve_path(&image.content_url, ext);
            if path.exists() {
                self.process_image(&path, image);
                return;
            }
        }

        tracing::debug!("Image file does not exist: {}", image.content_url);
    }

    /// Process an image file by optimizing and converting to data URI
    fn process_image(&self, path: &Path, image: &mut ImageObject) {
        // Check if this is a PDF file and handle it separately
        if let Some(extension) = path.extension()
            && extension.to_string_lossy().to_lowercase() == "pdf"
        {
            self.process_pdf_image(path, image);
            return;
        }

        // Determine input format
        let input_format = match ImageFormat::from_path(path) {
            Ok(format) => format,
            Err(error) => {
                tracing::error!("Failed to determine image format: {error}");
                return;
            }
        };

        // Load the image
        let img = match ImageReader::open(path) {
            Ok(reader) => match reader.decode() {
                Ok(img) => img,
                Err(error) => {
                    tracing::error!("Failed to decode image: {error}");
                    return;
                }
            },
            Err(error) => {
                tracing::error!("Failed to open image: {error}");
                return;
            }
        };
        let (original_width, original_height) = img.dimensions();

        // Determine the output format based on image_format field or input format
        let output_format = if let Some(desired_format) = &self.image_format {
            match desired_format {
                Format::Png => ImageFormat::Png,
                Format::Jpeg => ImageFormat::Jpeg,
                Format::Gif => ImageFormat::Gif,
                Format::WebP => ImageFormat::WebP,
                _ => {
                    // If the desired format is not a supported image format, fallback to input format
                    // but convert TIFF to PNG for web compatibility
                    if input_format == ImageFormat::Tiff {
                        ImageFormat::Png
                    } else {
                        input_format
                    }
                }
            }
        } else {
            // No specific format requested, use input format but convert TIFF to PNG for web compatibility
            if input_format == ImageFormat::Tiff {
                ImageFormat::Png
            } else {
                input_format
            }
        };

        // Use shared resize function from images crate
        let resized = stencila_images::resize_image(img, &self.image_options);
        let new_dimensions = resized.dimensions();

        // Check if we need to convert format or if dimensions changed
        let needs_conversion = output_format != input_format;
        let was_resized = new_dimensions != (original_width, original_height);

        let data_uri = if !was_resized && !needs_conversion {
            // No changes needed: encode directly without processing
            let mut bytes: Vec<u8> = Vec::new();
            if let Err(error) = resized.write_to(&mut Cursor::new(&mut bytes), input_format) {
                tracing::error!("Failed to encode image: {error}");
                return;
            }
            let encoded = STANDARD.encode(&bytes);
            let mime_type = input_format.to_mime_type();
            format!("data:{mime_type};base64,{encoded}")
        } else if needs_conversion {
            // Explicit format conversion requested - honor the output_format
            let mut bytes: Vec<u8> = Vec::new();
            if let Err(error) = resized.write_to(&mut Cursor::new(&mut bytes), output_format) {
                tracing::error!("Failed to encode converted image: {error}");
                return;
            }
            let encoded = STANDARD.encode(&bytes);
            let mime_type = output_format.to_mime_type();
            format!("data:{mime_type};base64,{encoded}")
        } else {
            // Only resized, no format conversion - preserve original format
            // Use encode_image for PNG (applies grayscale optimization), otherwise use standard encoding
            let (bytes, mime_type) = if input_format == ImageFormat::Png {
                match self.image_options.encode_image(&resized) {
                    Ok(result) => result,
                    Err(error) => {
                        tracing::error!("Failed to encode image: {error}");
                        return;
                    }
                }
            } else {
                // Preserve original format (JPEG, GIF, WebP, etc.)
                let mut bytes: Vec<u8> = Vec::new();
                if let Err(error) = resized.write_to(&mut Cursor::new(&mut bytes), input_format) {
                    tracing::error!("Failed to encode resized image: {error}");
                    return;
                }
                (bytes, input_format.to_mime_type())
            };
            let encoded = STANDARD.encode(&bytes);
            format!("data:{mime_type};base64,{encoded}")
        };

        image.content_url = data_uri;
    }

    /// Process a PDF file by converting to JPEG and encoding as data URI
    fn process_pdf_image(&self, path: &Path, image: &mut ImageObject) {
        // Create a temporary file for the JPEG output
        let temp_file = match NamedTempFile::with_suffix(".jpg") {
            Ok(file) => file,
            Err(error) => {
                tracing::error!("Failed to create temporary file: {error}");
                return;
            }
        };
        let temp_output = temp_file.path();

        // Use ImageMagick convert to convert PDF to JPEG
        let mut command = Convert.command();
        command
            .args(["-density", "150"]) // 150 DPI for good quality, similar to 1200px max width
            .arg(format!("{}[0]", path.display())) // [0] means first page only
            .args(["-quality", "85"]) // 85% quality for good web compression
            .args(["-colorspace", "RGB"]) // Ensure consistent color space
            .arg(temp_output);

        match command.output() {
            Ok(output) => {
                if output.status.success() {
                    // Read the converted JPEG file and convert to base64
                    match std::fs::read(temp_output) {
                        Ok(jpeg_bytes) => {
                            let encoded = STANDARD.encode(&jpeg_bytes);
                            let data_uri = format!("data:image/jpeg;base64,{encoded}");
                            image.content_url = data_uri;
                        }
                        Err(error) => {
                            tracing::error!("Failed to read converted JPEG file: {error}");
                        }
                    }
                } else {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    tracing::error!(
                        "ImageMagick convert failed to process PDF. stdout: {stdout}, stderr: {stderr}"
                    );
                }

                // Temporary file will be cleaned up automatically when temp_file is dropped
            }
            Err(error) => {
                tracing::error!("Failed to execute ImageMagick convert: {error}");
            }
        }
    }

    /// Embed an audio file by converting to MP3 and encoding as data URI
    fn embed_audio(&self, audio: &mut AudioObject) {
        if audio.content_url.starts_with("data:") || audio.content_url.starts_with("http") {
            return;
        }

        const AUDIO_EXTENSIONS: &[&str] = &["", ".mp3", ".wav", ".flac", ".ogg", ".aac", ".m4a"];

        // Try different audio file extensions
        for ext in AUDIO_EXTENSIONS {
            let path = self.resolve_path(&audio.content_url, ext);
            if path.exists() {
                self.process_audio(&path, audio);
                return;
            }
        }

        tracing::debug!("Audio file does not exist: {}", audio.content_url);
    }

    /// Process an audio file using FFmpeg to optimize and convert to data URI
    fn process_audio(&self, path: &Path, audio: &mut AudioObject) {
        // Create a temporary file for the optimized output
        let temp_file = match NamedTempFile::with_suffix(".mp3") {
            Ok(file) => file,
            Err(error) => {
                tracing::error!("Failed to create temporary file: {error}");
                return;
            }
        };
        let temp_output = temp_file.path();

        // Use FFmpeg to convert and optimize the audio
        let mut command = Ffmpeg.command();
        command
            .args(["-i"])
            .arg(path)
            .args([
                "-c:a", "mp3", // Use MP3 codec
                "-b:a", "128k", // Good quality for web playback
                "-ar", "44100", // Standard sample rate
                "-y",    // Overwrite output file
            ])
            .arg(temp_output);

        match command.output() {
            Ok(output) => {
                if output.status.success() {
                    // Read the optimized audio file and convert to base64
                    match std::fs::read(temp_output) {
                        Ok(audio_bytes) => {
                            let encoded = STANDARD.encode(&audio_bytes);
                            let data_uri = format!("data:audio/mpeg;base64,{encoded}");
                            audio.content_url = data_uri;
                        }
                        Err(error) => {
                            tracing::error!("Failed to read optimized audio file: {error}");
                        }
                    }
                } else {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    tracing::error!(
                        "FFmpeg failed to process audio. stdout: {stdout}, stderr: {stderr}"
                    );
                }

                // Temporary file will be cleaned up automatically when temp_file is dropped
            }
            Err(error) => {
                tracing::error!("Failed to execute FFmpeg: {error}");
            }
        }
    }

    /// Embed a video by converting to MP4 and encoding as data URI
    fn embed_video(&self, video: &mut VideoObject) {
        if video.content_url.starts_with("data:") || video.content_url.starts_with("http") {
            return;
        }

        const VIDEO_EXTENSIONS: &[&str] = &["", ".mp4", ".avi", ".mov", ".mkv", ".webm", ".wmv"];

        // Try different video file extensions
        for ext in VIDEO_EXTENSIONS {
            let path = self.resolve_path(&video.content_url, ext);
            if path.exists() {
                self.process_video(&path, video);
                return;
            }
        }

        tracing::debug!("Video file does not exist: {}", video.content_url);
    }

    /// Process a video file using FFmpeg to optimize and convert to data URI
    fn process_video(&self, path: &Path, video: &mut VideoObject) {
        // Create a temporary file for the optimized output
        let temp_file = match NamedTempFile::with_suffix(".mp4") {
            Ok(file) => file,
            Err(error) => {
                tracing::error!("Failed to create temporary file: {error}");
                return;
            }
        };
        let temp_output = temp_file.path();

        // Use FFmpeg to convert and optimize the video
        let mut command = Ffmpeg.command();
        command
            .args(["-i"])
            .arg(path)
            .args([
                "-c:v",
                "libx264", // Use H.264 codec
                "-preset",
                "medium", // Better quality/compression balance
                "-crf",
                "23", // Good quality for web viewing
                "-vf",
                "scale=-2:min(720\\,ih)", // Scale to max 720p height for desktop viewing
                "-r",
                "24", // Smooth playback framerate
                "-c:a",
                "aac", // Use AAC audio codec
                "-b:a",
                "128k", // Good audio quality for web
                "-movflags",
                "+faststart", // Enable fast start for web playback
                "-y",         // Overwrite output file
            ])
            .arg(temp_output);

        match command.output() {
            Ok(output) => {
                if output.status.success() {
                    // Read the optimized video file and convert to base64
                    match std::fs::read(temp_output) {
                        Ok(video_bytes) => {
                            let encoded = STANDARD.encode(&video_bytes);
                            let data_uri = format!("data:video/mp4;base64,{}", encoded);
                            video.content_url = data_uri;
                        }
                        Err(error) => {
                            tracing::error!("Failed to read optimized video file: {error}");
                        }
                    }
                } else {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    tracing::error!(
                        "FFmpeg failed to process video. stdout: {stdout}, stderr: {stderr}"
                    );
                }

                // Temporary file will be cleaned up automatically when temp_file is dropped
            }
            Err(error) => {
                tracing::error!("Failed to execute FFmpeg: {error}");
            }
        }
    }
}

impl VisitorMut for Embedder {
    fn visit_node(&mut self, node: &mut Node) -> WalkControl {
        match node {
            Node::AudioObject(audio) => self.embed_audio(audio),
            Node::ImageObject(image) => self.embed_image(image),
            Node::VideoObject(video) => self.embed_video(video),
            Node::MathBlock(math) => {
                if let Some(images) = &mut math.options.images {
                    self.embed_images(images)
                }
            }
            Node::MathInline(math) => {
                if let Some(images) = &mut math.options.images {
                    self.embed_images(images)
                }
            }
            Node::Table(table) => {
                if let Some(images) = &mut table.options.images {
                    self.embed_images(images)
                }
            }
            _ => {}
        }
        WalkControl::Continue
    }

    fn visit_work(&mut self, work: &mut CreativeWorkVariant) -> WalkControl {
        match work {
            CreativeWorkVariant::AudioObject(audio) => self.embed_audio(audio),
            CreativeWorkVariant::ImageObject(image) => self.embed_image(image),
            CreativeWorkVariant::VideoObject(video) => self.embed_video(video),
            CreativeWorkVariant::Table(table) => {
                if let Some(images) = &mut table.options.images {
                    self.embed_images(images)
                }
            }
            _ => {}
        }
        WalkControl::Continue
    }

    fn visit_block(&mut self, block: &mut Block) -> WalkControl {
        match block {
            Block::AudioObject(audio) => self.embed_audio(audio),
            Block::ImageObject(image) => self.embed_image(image),
            Block::VideoObject(video) => self.embed_video(video),
            Block::MathBlock(math) => {
                if let Some(images) = &mut math.options.images {
                    self.embed_images(images)
                }
            }
            Block::Table(table) => {
                if let Some(images) = &mut table.options.images {
                    self.embed_images(images)
                }
            }
            _ => {}
        }
        WalkControl::Continue
    }

    fn visit_inline(&mut self, inline: &mut Inline) -> WalkControl {
        match inline {
            Inline::AudioObject(audio) => self.embed_audio(audio),
            Inline::ImageObject(image) => self.embed_image(image),
            Inline::VideoObject(video) => self.embed_video(video),
            Inline::MathInline(math) => {
                if let Some(images) = &mut math.options.images {
                    self.embed_images(images)
                }
            }
            _ => {}
        }
        WalkControl::Continue
    }
}
