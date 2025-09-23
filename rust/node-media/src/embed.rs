use std::{
    env::current_dir,
    io::Cursor,
    path::{Path, PathBuf},
};

use base64::{Engine as _, engine::general_purpose::STANDARD};
use eyre::{Context, Result, bail, eyre};
use image::{GenericImageView, ImageFormat, ImageReader, imageops};
use stencila_format::Format;
use tempfile::NamedTempFile;

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
pub fn embed_media<T>(node: &mut T, path: Option<&Path>) -> Result<()>
where
    T: WalkNode,
{
    let mut embedder = Embedder::new(path, None)?;
    embedder.walk(node);

    Ok(())
}

/// Embed an in individual image object
///
/// Use this when you want to embed an individual image, rather than all media
/// nested within some other node.
pub fn embed_image(
    image: &mut ImageObject,
    path: Option<&Path>,
    format: Option<Format>,
) -> Result<()> {
    let embedder = Embedder::new(path, format)?;
    embedder.embed_image(image);

    Ok(())
}

/// Embed an in individual audio object
///
/// Use this when you want to embed an individual audio, rather than all media
/// nested within some other node.
pub fn embed_audio(audio: &mut AudioObject, path: Option<&Path>) -> Result<()> {
    let embedder = Embedder::new(path, None)?;
    embedder.embed_audio(audio);

    Ok(())
}

/// Embed an in individual video object
///
/// Use this when you want to embed an individual video, rather than all media
/// nested within some other node.
pub fn embed_video(video: &mut VideoObject, path: Option<&Path>) -> Result<()> {
    let embedder = Embedder::new(path, None)?;
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
}

impl Embedder {
    fn new(path: Option<&Path>, image_format: Option<Format>) -> Result<Self> {
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

        Ok(Self { dir, image_format })
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

        const MAX_WIDTH: u32 = 1200; // Default max width for web viewing

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

        // Check if we need to resize or convert format
        let needs_resize = original_width > MAX_WIDTH;
        let needs_conversion = output_format != input_format;

        let data_uri = if !needs_resize && !needs_conversion {
            // Small image in correct format: convert directly without processing
            let mut bytes: Vec<u8> = Vec::new();
            if let Err(error) = img.write_to(&mut Cursor::new(&mut bytes), input_format) {
                tracing::error!("Failed to encode image: {error}");
                return;
            }
            let encoded = STANDARD.encode(&bytes);
            let mime_type = input_format.to_mime_type();
            format!("data:{mime_type};base64,{encoded}")
        } else {
            // Need to process the image (resize and/or convert format)

            // Calculate new dimensions for resizing if needed
            let (new_width, new_height) = if needs_resize {
                // Calculate proportional height to maintain aspect ratio
                let aspect_ratio = original_height as f64 / original_width as f64;
                let new_height = (MAX_WIDTH as f64 * aspect_ratio).round() as u32;
                (MAX_WIDTH, new_height)
            } else {
                (original_width, original_height)
            };

            // Resize the image if dimensions changed
            let processed_img = if (new_width, new_height) != (original_width, original_height) {
                imageops::resize(&img, new_width, new_height, imageops::FilterType::Lanczos3)
            } else {
                img.to_rgba8()
            };

            // Convert to DynamicImage
            let dynamic_img = image::DynamicImage::ImageRgba8(processed_img);

            let mime_type = output_format.to_mime_type();

            // Convert to data URI
            let mut bytes: Vec<u8> = Vec::new();
            if let Err(error) = dynamic_img.write_to(&mut Cursor::new(&mut bytes), output_format) {
                tracing::error!("Failed to encode processed image: {error}");
                return;
            }
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
