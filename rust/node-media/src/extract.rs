use std::{
    fs::{File, create_dir_all},
    hash::{Hash, Hasher},
    io::Write,
    path::{Path, PathBuf},
};

use base64::{Engine as _, engine::general_purpose::STANDARD};
use eyre::{OptionExt, Result, bail};
use itertools::Itertools;
use seahash::SeaHasher;

use stencila_format::Format;
use stencila_schema::{
    AudioObject, Block, CreativeWorkVariant, ImageObject, Inline, Node, VideoObject, VisitorMut,
    WalkControl, WalkNode,
};

/// Extract any [`ImageObject`], [`AudioObject`], and [`VideoObject`] with dataURIs to files
/// and change their content_url to point to the extracted files
///
/// This function processes all media objects in the document tree, extracting embedded
/// data URIs to the specified directory and updating the objects to reference the
/// extracted files instead.
///
/// See the `media-embed` crate for doing the opposite: embedding
/// files as dataURIs.
pub fn extract_media<T>(node: &mut T, dir: &Path) -> Result<()>
where
    T: WalkNode,
{
    let mut walker = Extractor { dir: dir.into() };
    walker.walk(node);

    Ok(())
}

struct Extractor {
    /// The directory where media files will be written
    dir: PathBuf,
}

/// Convert a data URI into a media file
///
/// The media will be converted into a file with a name based on the hash of the
/// URI and an extension based on the MIME type of the data URI.
///
/// # Arguments
///
/// - `data_uri`: the data URI
/// - `media_dir`: the destination media directory
///
/// # Returns
///
/// The full path of the created media file.
fn data_uri_to_file(data_uri: &str, media_dir: &Path) -> Result<String> {
    // Parse the data URI
    let Some((header, data)) = data_uri.split(',').collect_tuple() else {
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
        .map_err(|_| eyre::eyre!("Unsupported media format: {mime_type}"))?;

    let extension = if mime_type == "audio/mp4" {
        // Special case: audio/mp4 should use m4a extension
        "m4a".to_string()
    } else {
        format.extension()
    };

    // Decode the Base64 data
    let decoded_data = STANDARD.decode(data.as_bytes())?;

    // Generate a hash of the data URI
    let mut hash = SeaHasher::new();
    data_uri.hash(&mut hash);
    let hash = hash.finish();
    let media_name = format!("{hash:x}.{extension}");

    // Ensure the media directory exists
    if !media_dir.exists() {
        create_dir_all(media_dir)?;
    }

    // Create the full file path
    let full_path = media_dir.join(&media_name);

    // Write the decoded data to the file
    let mut file = File::create(&full_path)?;
    file.write_all(&decoded_data)?;

    Ok(full_path.to_string_lossy().to_string())
}

impl Extractor {
    fn extract_images(&self, images: &mut [ImageObject]) {
        images
            .iter_mut()
            .for_each(|image| self.extract_image(image));
    }

    fn extract_image(&self, image: &mut ImageObject) {
        if image.content_url.starts_with("data:") {
            match data_uri_to_file(&image.content_url, &self.dir) {
                Ok(file_path) => {
                    image.content_url = file_path;
                }
                Err(error) => tracing::error!("While writing image to file: {error}"),
            }
        }
    }

    fn extract_audio(&self, audio: &mut AudioObject) {
        if audio.content_url.starts_with("data:") {
            match data_uri_to_file(&audio.content_url, &self.dir) {
                Ok(file_path) => {
                    audio.content_url = file_path;
                }
                Err(error) => tracing::error!("While writing audio to file: {error}"),
            }
        }
    }

    fn extract_video(&self, video: &mut VideoObject) {
        if video.content_url.starts_with("data:") {
            match data_uri_to_file(&video.content_url, &self.dir) {
                Ok(file_path) => {
                    video.content_url = file_path;
                }
                Err(error) => tracing::error!("While writing video to file: {error}"),
            }
        }
    }
}

impl VisitorMut for Extractor {
    fn visit_node(&mut self, node: &mut Node) -> WalkControl {
        match node {
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
