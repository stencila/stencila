use std::path::{Path, PathBuf};

use eyre::{Context, Result, bail, eyre};

use stencila_images::path_to_data_uri_to_embed;
use stencila_schema::{Block, ImageObject, Inline, Node, VisitorMut, WalkControl, WalkNode};

/// Embed any media files within [`ImageObject`] and other media objects as dataURIs
///
/// Currently only handles images but in the future may also support
/// audio and (small) video.
///
/// See the `media-extract` crate for doing the opposite: extracting
/// dataURIs to files.
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
    /// The directory containing images
    dir: PathBuf,
}

impl Walker {
    fn embed_image(&self, image: &mut ImageObject) {
        for ext in ["", ".png", ".jpg", ".jpeg", ".gif", ".tif", ".tiff"] {
            let path = self.dir.join([&image.content_url, ext].concat());
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
}

impl VisitorMut for Walker {
    fn visit_node(&mut self, node: &mut Node) -> WalkControl {
        if let Node::ImageObject(image) = node {
            self.embed_image(image)
        }

        WalkControl::Continue
    }

    fn visit_block(&mut self, block: &mut Block) -> WalkControl {
        if let Block::ImageObject(image) = block {
            self.embed_image(image)
        }

        WalkControl::Continue
    }

    fn visit_inline(&mut self, inline: &mut Inline) -> WalkControl {
        if let Inline::ImageObject(image) = inline {
            self.embed_image(image)
        }

        WalkControl::Continue
    }
}
