use std::{
    fs::create_dir_all,
    path::{Path, PathBuf},
};

use eyre::{Context, Result, bail, eyre};

use stencila_codec::{Codec, EncodeOptions, stencila_format::Format};
use stencila_codec_cbor::CborCodec;
use stencila_node_media::extract_media;
use stencila_schema::{Block, Node, Supplement, VisitorAsync, WalkControl, WalkNode};

/// Extract any [`Supplement`] nodes within a node
pub async fn extract_supplements<T: WalkNode>(node: &mut T, path: &Path) -> Result<()> {
    let path = path
        .canonicalize()
        .wrap_err_with(|| eyre!("Path does not exist: {}", path.display()))?;

    let dir = if path.is_file()
        && let Some(parent) = path.parent()
    {
        parent.to_path_buf()
    } else {
        path
    };

    if !dir.exists() {
        bail!("Directory does not exist: {}", dir.display());
    }

    let mut extractor = Extractor { dir, counter: 0 };
    node.walk_async(&mut extractor).await?;

    Ok(())
}

struct Extractor {
    /// The directory into which supplements, including media, are written
    dir: PathBuf,

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
        if !self.dir.exists() {
            create_dir_all(&self.dir)?;
        }

        // Increment counter for media & filenames below
        self.counter += 1;

        // Extract any media
        let mut work: Node = work.clone().into();
        extract_media(
            &mut work,
            &self.dir.join(format!("supplement-{}.media", self.counter)),
        )?;

        // Write the supplement itself
        CborCodec
            .to_path(
                &work,
                &self
                    .dir
                    .join(format!("supplement-{}.cbor.zstd", self.counter)),
                Some(EncodeOptions {
                    format: Some(Format::CborZstd),
                    ..Default::default()
                }),
            )
            .await?;

        Ok(())
    }
}
