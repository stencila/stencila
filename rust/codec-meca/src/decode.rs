use std::path::{Path, PathBuf};

use codec::{
    common::{
        eyre::{OptionExt, Result},
        glob::glob,
        tempfile, tracing,
    },
    schema::{Block, ImageObject, Inline, Node, VisitorMut, WalkControl, WalkNode},
    Codec, DecodeInfo, DecodeOptions,
};
use codec_jats::JatsCodec;
use zip::ZipArchive;

/// Decode a MECA file to a Stencila [`Node`]
#[tracing::instrument]
pub(super) async fn decode_path(
    path: &Path,
    options: Option<DecodeOptions>,
) -> Result<(Node, Option<Node>, DecodeInfo)> {
    // Create temporary directory to extract into
    // if path is not already a directory (e.g. an unzipped MECA)
    let tempdir = tempfile::TempDir::new()?;
    let dir = if path.is_dir() { path } else { tempdir.path() };

    if path.is_file() {
        tracing::debug!("Extracting MECA");
        let file = std::fs::File::open(path)?;
        let mut zip = ZipArchive::new(file)?;
        zip.extract(dir)?;
    }

    let dir = dir.join("content");

    // Find the JATS file in the dir
    let jats_path = glob(&dir.join("*.xml").to_string_lossy())?
        .next()
        .and_then(|res| res.ok())
        .ok_or_eyre("Unable to find JATS XML file")?;

    // Decode the JATS
    let (mut node, .., info) = JatsCodec.from_path(&jats_path, options).await?;

    // Inline any images if possible
    node.walk_mut(&mut ImageInliner { dir });

    Ok((node, None, info))
}

/// Reads any image files in the package and "inlines" them into the node's
/// `content_url` as a dataURI
struct ImageInliner {
    dir: PathBuf,
}

impl ImageInliner {
    fn inline_image(&self, image: &mut ImageObject) {
        for ext in ["", ".png", ".jpg", ".jpeg", ".gif", ".tif", ".tiff"] {
            let mut path = self.dir.join([&image.content_url, ext].concat());

            if path.exists() {
                if matches!(
                    path.extension().and_then(|ext| ext.to_str()),
                    Some("tif" | "tiff")
                ) {
                    let mut to = path.clone();
                    to.set_extension("png");

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

impl VisitorMut for ImageInliner {
    fn visit_block(&mut self, block: &mut Block) -> WalkControl {
        if let Block::ImageObject(image) = block {
            self.inline_image(image)
        }

        WalkControl::Continue
    }

    fn visit_inline(&mut self, inline: &mut Inline) -> WalkControl {
        if let Inline::ImageObject(image) = inline {
            self.inline_image(image)
        }

        WalkControl::Continue
    }
}
