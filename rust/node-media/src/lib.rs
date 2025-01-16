use std::path::{Path, PathBuf};

use common::tracing;
use schema::{Block, ImageObject, Inline, VisitorMut, WalkControl, WalkNode};

/// Walk over a node and extract/copy all media files into a folder and re-write URLs
///
/// # Arguments
///
/// `src_dir`: The directory of the node being walked, used to resolve any relative
///            paths to media files in the node
///
/// `dest_dir`: The destination directory that collected media files should be placed in
///
/// `rewrite`: A function, usually a closure, to rewrite the URL of media nodes
///            (i.e. images, audio, and video) within the root node. This function
///            should have arguments (old_url: &str, media_file_name: &str) and return
///            a new URL as a `String`.
pub fn extract_media<T, F>(node: &mut T, src_dir: &Path, dest_dir: &Path, rewrite: F)
where
    T: WalkNode,
    F: Fn(&str, &str) -> String,
{
    let mut walker = Walker {
        src_dir: src_dir.to_path_buf(),
        dest_dir: dest_dir.to_path_buf(),
        rewrite,
    };
    walker.visit(node);
}

/// A visitor that collects node ids and addresses
struct Walker<F>
where
    F: Fn(&str, &str) -> String,
{
    src_dir: PathBuf,
    dest_dir: PathBuf,
    rewrite: F,
}

impl<F> Walker<F>
where
    F: Fn(&str, &str) -> String,
{
    /// Visit an image to extract it and rewrite its URL
    fn visit_image(&mut self, image: &mut ImageObject) -> WalkControl {
        let image_path = if image.content_url.starts_with("data:") {
            // Encode the data URI to a file
            match images::data_uri_to_file(&image.content_url, &self.dest_dir) {
                Ok(path) => Some(path),
                Err(error) => {
                    tracing::warn!("While encoding image data URI to file: {error}");
                    None
                }
            }
        } else {
            match images::file_uri_to_file(&image.content_url, Some(&self.src_dir), &self.dest_dir)
            {
                Ok(path) => Some(path),
                Err(error) => {
                    tracing::warn!("While encoding image to file: {error}");
                    None
                }
            }
        };

        if let Some(image_path) = image_path {
            image.content_url = (self.rewrite)(&image.content_url, &image_path);
        }

        WalkControl::Break
    }
}

impl<F> VisitorMut for Walker<F>
where
    F: Fn(&str, &str) -> String,
{
    /// Visit a `Block` node type
    fn visit_block(&mut self, block: &mut Block) -> WalkControl {
        match block {
            Block::ImageObject(image) => self.visit_image(image),
            _ => WalkControl::Continue,
        }
    }

    /// Visit an `Inline` node type
    fn visit_inline(&mut self, inline: &mut Inline) -> WalkControl {
        match inline {
            Inline::ImageObject(image) => self.visit_image(image),
            _ => WalkControl::Continue,
        }
    }
}
