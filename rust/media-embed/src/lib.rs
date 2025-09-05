use std::path::{Path, PathBuf};

use base64::{Engine as _, engine::general_purpose::STANDARD};
use eyre::{Context, Result, bail, eyre};

use stencila_images::path_to_data_uri_to_embed;
use stencila_schema::{
    AudioObject, Block, ImageObject, Inline, Node, VideoObject, VisitorMut, WalkControl, WalkNode,
};
use stencila_tools::{Ffmpeg, Tool};

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
pub fn embed_media<T>(node: &mut T, path: &Path) -> Result<()>
where
    T: WalkNode,
{
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

    let mut walker = Walker { dir: dir.into() };
    walker.walk(node);

    Ok(())
}

struct Walker {
    /// The base directory for relative filesystem paths
    dir: PathBuf,
}

impl Walker {
    /// Resolve a media file path, handling both absolute and relative paths
    fn resolve_media_path(&self, content_url: &str, extension: &str) -> PathBuf {
        let path_with_ext = [content_url, extension].concat();
        let path = Path::new(&path_with_ext);

        if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.dir.join(path)
        }
    }

    /// Embed an image by converting to a data URI using optimized settings
    fn embed_image(&self, image: &mut ImageObject) {
        for ext in ["", ".png", ".jpg", ".jpeg", ".gif", ".tif", ".tiff"] {
            let path = self.resolve_media_path(&image.content_url, ext);
            if path.exists() {
                match path_to_data_uri_to_embed(&path, None) {
                    Ok(url) => {
                        image.content_url = url;
                    }
                    Err(error) => {
                        tracing::error!("While converting image to dataURI: {error}")
                    }
                }

                return;
            }
        }
    }

    /// Embed an audio file by converting to MP3 and encoding as data URI
    fn embed_audio(&self, audio: &mut AudioObject) {
        // Try different audio file extensions
        for ext in ["", ".mp3", ".wav", ".flac", ".ogg", ".aac", ".m4a"] {
            let path = self.resolve_media_path(&audio.content_url, ext);
            if path.exists() {
                self.process_audio_file(&path, audio);
                return;
            }
        }

        tracing::warn!("Audio file does not exist: {}", audio.content_url);
    }

    /// Process an audio file using FFmpeg to optimize and convert to data URI
    fn process_audio_file(&self, path: &Path, audio: &mut AudioObject) {
        // Create a temporary file for the optimized output
        let temp_dir = std::env::temp_dir();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);
        let temp_output = temp_dir.join(format!("embedded_{}.mp3", timestamp));

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
            .arg(&temp_output);

        match command.output() {
            Ok(output) => {
                if output.status.success() {
                    // Read the optimized audio file and convert to base64
                    match std::fs::read(&temp_output) {
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
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    tracing::error!("FFmpeg failed to process audio: {stderr}");
                }

                // Clean up temporary file
                if temp_output.exists()
                    && let Err(error) = std::fs::remove_file(&temp_output)
                {
                    tracing::warn!("Failed to remove temporary file: {error}");
                }
            }
            Err(error) => {
                tracing::error!("Failed to execute FFmpeg: {error}");
            }
        }
    }

    /// Embed a video by converting to MP4 and encoding as data URI
    fn embed_video(&self, video: &mut VideoObject) {
        // Try different video file extensions
        for ext in ["", ".mp4", ".avi", ".mov", ".mkv", ".webm", ".wmv"] {
            let path = self.resolve_media_path(&video.content_url, ext);
            if path.exists() {
                self.process_video_file(&path, video);
                return;
            }
        }

        tracing::warn!("Video file does not exist: {}", video.content_url);
    }

    /// Process a video file using FFmpeg to optimize and convert to data URI
    fn process_video_file(&self, path: &Path, video: &mut VideoObject) {
        // Create a temporary file for the optimized output
        let temp_dir = std::env::temp_dir();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);
        let temp_output = temp_dir.join(format!("embedded_{}.mp4", timestamp));

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
            .arg(&temp_output);

        match command.output() {
            Ok(output) => {
                if output.status.success() {
                    // Read the optimized video file and convert to base64
                    match std::fs::read(&temp_output) {
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
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    tracing::error!("FFmpeg failed to process video: {stderr}");
                }

                // Clean up temporary file
                if temp_output.exists()
                    && let Err(error) = std::fs::remove_file(&temp_output)
                {
                    tracing::warn!("Failed to remove temporary file: {error}");
                }
            }
            Err(error) => {
                tracing::error!("Failed to execute FFmpeg: {error}");
            }
        }
    }
}

impl VisitorMut for Walker {
    fn visit_node(&mut self, node: &mut Node) -> WalkControl {
        match node {
            Node::AudioObject(audio) => self.embed_audio(audio),
            Node::ImageObject(image) => self.embed_image(image),
            Node::VideoObject(video) => self.embed_video(video),
            _ => {}
        }
        WalkControl::Continue
    }

    fn visit_block(&mut self, block: &mut Block) -> WalkControl {
        match block {
            Block::AudioObject(audio) => self.embed_audio(audio),
            Block::ImageObject(image) => self.embed_image(image),
            Block::VideoObject(video) => self.embed_video(video),
            _ => {}
        }
        WalkControl::Continue
    }

    fn visit_inline(&mut self, inline: &mut Inline) -> WalkControl {
        match inline {
            Inline::AudioObject(audio) => self.embed_audio(audio),
            Inline::ImageObject(image) => self.embed_image(image),
            Inline::VideoObject(video) => self.embed_video(video),
            _ => {}
        }
        WalkControl::Continue
    }
}
