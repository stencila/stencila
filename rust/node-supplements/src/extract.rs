use std::{
    fs::create_dir_all,
    path::{Path, PathBuf},
};

use eyre::Result;

use pathdiff::diff_paths;
use stencila_codec::{Codec, EncodeOptions, stencila_format::Format};
use stencila_codec_cbor::CborCodec;
use stencila_schema::{Block, Node, Supplement, VisitorAsync, WalkControl, WalkNode};

/// Extract any [`Supplement`] nodes within a node
#[tracing::instrument(skip(node))]
pub async fn extract_supplements<T: WalkNode>(
    node: &mut T,
    document_path: &Path,
    supplements_dir: &Path,
) -> Result<()> {
    let mut extractor = Extractor {
        document_path: document_path.into(),
        supplements_dir: supplements_dir.into(),
        counter: 0,
    };
    node.walk_async(&mut extractor).await?;

    Ok(())
}

struct Extractor {
    /// The path of the document. Used to determine relative paths to the extracted supplements.
    document_path: PathBuf,

    /// The directory into which supplements, including their media, are written
    supplements_dir: PathBuf,

    /// A counter of the supplements used to create unique filenames
    counter: u32,
}

impl VisitorAsync for Extractor {
    async fn visit_node(&mut self, node: &mut Node) -> Result<WalkControl> {
        if let Node::Supplement(supplement) = node {
            self.extract(supplement).await?;
            Ok(WalkControl::Break)
        } else {
            Ok(WalkControl::Continue)
        }
    }

    async fn visit_block(&mut self, block: &mut Block) -> Result<WalkControl> {
        if let Block::Supplement(supplement) = block {
            self.extract(supplement).await?;
            Ok(WalkControl::Break)
        } else {
            Ok(WalkControl::Continue)
        }
    }
}

impl Extractor {
    /// Extract a supplement
    ///
    /// Extracts all supplements using `supplement-<index>.cbor+zstd`. Media
    /// within each supplement are extracted to media subdirectories.
    async fn extract(&mut self, supplement: &mut Supplement) -> Result<()> {
        let Some(work) = &supplement.options.work else {
            return Ok(());
        };

        // Ensure the supplements directory exists
        if !self.supplements_dir.exists() {
            create_dir_all(&self.supplements_dir)?;
        }

        // Increment counter
        self.counter += 1;

        // Determine file and directory names
        let stem = format!("supplement-{}", self.counter);
        let path = self.supplements_dir.join(format!("{stem}.czst"));
        let media = self.supplements_dir.join(format!("{stem}.media"));

        // Write the supplement
        let node = work.clone().into();
        CborCodec
            .to_path(
                &node,
                &path,
                Some(EncodeOptions {
                    format: Some(Format::CborZstd),
                    extract_media: Some(media),
                    ..Default::default()
                }),
            )
            .await?;

        // Set work to None
        supplement.options.work = None;

        // Update the target to the relative path of the extracted file
        let relative_path = self
            .document_path
            .parent()
            .and_then(|base| diff_paths(&path, &base))
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();
        supplement.target = Some(relative_path);

        Ok(())
    }
}
