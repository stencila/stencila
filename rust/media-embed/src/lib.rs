use std::path::{Path, PathBuf};

use eyre::{Context, Result, bail, eyre};
use format::Format;
use schema::{Block, ImageObject, Inline, VisitorMut, WalkControl, WalkNode};

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

    let mut walker = Walker {
        dir: dir.into(),
        tiff_to: Some(Format::Png),
    };
    walker.walk(node);

    Ok(())
}

struct Walker {
    /// The directory containing images
    dir: PathBuf,

    /// The format to convert TIFF images to
    tiff_to: Option<Format>,
}

impl Walker {
    fn embed_image(&self, image: &mut ImageObject) {
        for ext in ["", ".png", ".jpg", ".jpeg", ".gif", ".tif", ".tiff"] {
            let mut path = self.dir.join([&image.content_url, ext].concat());

            if path.exists() {
                if let (Some("tif" | "tiff"), Some(format)) =
                    (path.extension().and_then(|ext| ext.to_str()), &self.tiff_to)
                {
                    let mut to = path.clone();
                    to.set_extension(format.extension());

                    match images::convert(&path, &to) {
                        Ok(..) => {
                            path = to;
                        }
                        Err(error) => {
                            tracing::error!("While converting TIFF to PNG: {error}")
                        }
                    }
                }

                match images::path_to_data_uri(&path) {
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
