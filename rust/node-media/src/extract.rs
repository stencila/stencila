use std::{
    env::current_dir,
    fs::{File, create_dir_all},
    hash::{Hash, Hasher},
    io::Write,
    path::{Path, PathBuf},
};

use base64::{Engine as _, engine::general_purpose::STANDARD};
use eyre::{OptionExt, Result, bail, eyre};
use itertools::Itertools;
use pathdiff::diff_paths;
use seahash::SeaHasher;

use stencila_codec_info::EncodedAsset;
use stencila_format::Format;
use stencila_schema::{
    AudioObject, Block, CreativeWorkVariant, ImageObject, Inline, Node, NodeId, NodeType,
    VideoObject, VisitorMut, WalkControl, WalkNode,
};

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
    fn data_uri_to_file(&mut self, data_uri: &str) -> Result<(PathBuf, String)> {
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
            .map_err(|_| eyre!("Unsupported media format: {mime_type}"))?;

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
        if !self.media_dir.exists() {
            create_dir_all(&self.media_dir)?;
        }

        // Create the full file path
        let path = self.media_dir.join(&media_name);

        // Write the decoded data to the file
        let mut file = File::create(&path)?;
        file.write_all(&decoded_data)?;

        let relative_path = diff_paths(&path, &self.document_dir)
            .unwrap_or_else(|| path.clone())
            .to_string_lossy()
            .to_string();

        Ok((path, relative_path))
    }

    /// Record an extracted asset with originating-node attribution.
    fn record_asset(&mut self, path: PathBuf, self_id: Option<&NodeId>, self_type: NodeType) {
        let (node_type, node_id) = self.originating(self_id, self_type);
        let role = role_for(node_type);
        self.assets.push(EncodedAsset {
            path,
            node_id: node_id.map(|id| id.to_string()),
            node_type: Some(node_type.to_string()),
            role: Some(role.to_string()),
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

    fn extract_images(&mut self, images: &mut [ImageObject]) {
        images
            .iter_mut()
            .for_each(|image| self.extract_image(image));
    }

    fn extract_image(&mut self, image: &mut ImageObject) {
        if image.content_url.starts_with("data:") {
            match self.data_uri_to_file(&image.content_url) {
                Ok((path, file_path)) => {
                    let id = image.node_id();
                    self.record_asset(path, Some(&id), NodeType::ImageObject);
                    image.content_url = file_path;
                }
                Err(error) => tracing::error!("While writing image to file: {error}"),
            }
        }
    }

    fn extract_audio(&mut self, audio: &mut AudioObject) {
        if audio.content_url.starts_with("data:") {
            match self.data_uri_to_file(&audio.content_url) {
                Ok((path, file_path)) => {
                    let id = audio.node_id();
                    self.record_asset(path, Some(&id), NodeType::AudioObject);
                    audio.content_url = file_path;
                }
                Err(error) => tracing::error!("While writing audio to file: {error}"),
            }
        }
    }

    fn extract_video(&mut self, video: &mut VideoObject) {
        if video.content_url.starts_with("data:") {
            match self.data_uri_to_file(&video.content_url) {
                Ok((path, file_path)) => {
                    let id = video.node_id();
                    self.record_asset(path, Some(&id), NodeType::VideoObject);
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
