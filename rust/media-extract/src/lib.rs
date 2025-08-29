use std::path::{Path, PathBuf};

use common::{eyre::Result, tracing};
use schema::{Block, ImageObject, Inline, VisitorMut, WalkControl, WalkNode};

/// Write any [`ImageObject`] and other media objects with a dataURI to a file
/// and change target accordingly
///
/// Currently only handles images with dataURIs but in the future may also
/// support audio and video and collection of files from the file system into
/// the directory.
///
/// See the `media-embed` crate for doing the opposite: embedding
/// files as dataURIs.
pub fn extract_media<T>(node: &mut T, dir: &Path) -> Result<()>
where
    T: WalkNode,
{
    let mut walker = Walker { dir: dir.into() };
    walker.walk(node);

    Ok(())
}

struct Walker {
    /// The directory where images will be written
    dir: PathBuf,
}

impl Walker {
    fn extract_image(&self, image: &mut ImageObject) {
        if image.content_url.starts_with("data:image/") {
            match images::data_uri_to_file(&image.content_url, &self.dir) {
                Ok(file_name) => {
                    image.content_url = self.dir.join(file_name).to_string_lossy().to_string();
                }
                Err(error) => tracing::error!("While writing image to file: {error}"),
            }
        }
    }
}

impl VisitorMut for Walker {
    fn visit_block(&mut self, block: &mut Block) -> WalkControl {
        if let Block::ImageObject(image) = block {
            self.extract_image(image)
        }

        WalkControl::Continue
    }

    fn visit_inline(&mut self, inline: &mut Inline) -> WalkControl {
        if let Inline::ImageObject(image) = inline {
            self.extract_image(image)
        }

        WalkControl::Continue
    }
}
