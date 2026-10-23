use std::path::{Path, PathBuf};

use derive_more::{Deref, DerefMut};
use roxmltree::Node;
use stencila_codec::Losses;
use stencila_codec_jats_trait::has_uri_scheme;

use super::utilities::XLINK;

/// File context is present only for path decoding. String decoding does not
/// assume that relative references name files in the process's working directory.
#[derive(Default, Deref, DerefMut)]
pub(super) struct DecodeContext {
    #[deref]
    #[deref_mut]
    pub losses: Losses,
    directory: Option<PathBuf>,
}

impl DecodeContext {
    pub fn new(directory: Option<&Path>) -> Self {
        Self {
            directory: directory.map(Path::to_path_buf),
            ..Default::default()
        }
    }

    /// Resolve an exact file first, then files whose stem is the entire href
    /// basename. Prefer vector originals, then the greatest raster pixel area,
    /// then lossless formats. Lexical filename order breaks remaining ties.
    pub fn media(&mut self, node: &Node) -> (String, Option<String>) {
        let href = node.attribute((XLINK, "href")).unwrap_or_default();
        let Some(directory) = &self.directory else {
            return (href.into(), None);
        };
        if href.is_empty()
            || href.starts_with('#')
            || href.starts_with("//")
            || has_uri_scheme(href)
        {
            return (href.into(), None);
        }

        let exact = directory.join(href);
        let found = if exact.is_file() {
            Some(exact)
        } else {
            exact.parent().and_then(|parent| {
                let mut candidates = std::fs::read_dir(parent)
                    .ok()?
                    .filter_map(Result::ok)
                    .map(|entry| entry.path())
                    .filter(|path| path.is_file() && path.file_stem() == exact.file_name())
                    .map(|path| (preference(&path), path))
                    .collect::<Vec<_>>();
                candidates.sort_by(|(rank_a, path_a), (rank_b, path_b)| {
                    rank_b.cmp(rank_a).then_with(|| path_a.cmp(path_b))
                });
                candidates.into_iter().next().map(|(_, path)| path)
            })
        };

        match found {
            Some(path) => {
                let media_type = mime_guess::from_path(&path).first_raw().map(String::from);
                let reference = path.strip_prefix(directory).unwrap_or(&path);
                (reference.to_string_lossy().into(), media_type)
            }
            None => {
                // Include sibling indices and the original href so callers can
                // identify individual missing resources even in long articles.
                let mut segments = node
                    .ancestors()
                    .filter(Node::is_element)
                    .map(|node| {
                        let tag = node.tag_name().name();
                        let index = node
                            .prev_siblings()
                            .skip(1)
                            .filter(|sibling| sibling.has_tag_name(tag))
                            .count()
                            + 1;
                        format!("{tag}[{index}]")
                    })
                    .collect::<Vec<_>>();
                segments.reverse();
                self.losses.add(format!(
                    "//{}/@xlink:href (unresolved: {href})",
                    segments.join("/")
                ));
                (href.into(), None)
            }
        }
    }
}

fn preference(path: &Path) -> (bool, u64, u8) {
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    let vector = matches!(extension.as_str(), "svg" | "eps" | "pdf");
    let pixels = image::image_dimensions(path)
        .map(|(w, h)| u64::from(w) * u64::from(h))
        .unwrap_or_default();
    let format = match extension.as_str() {
        "svg" => 7,
        "eps" => 6,
        "pdf" => 5,
        "tif" | "tiff" => 4,
        "png" => 3,
        "jpg" | "jpeg" => 2,
        "gif" => 1,
        _ => 0,
    };
    (vector, pixels, format)
}
