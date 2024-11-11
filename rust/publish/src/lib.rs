use std::path::Path;

use codec::EncodeOptions;
use codec_swb::SwbCodec;
use common::eyre::{bail, Result};
use document::{CommandWait, Document};
use schema::Node;

pub mod cli;
mod stencila;

/// Publish a path (file or directory)
pub async fn publish_path(
    path: &Path,
    key: &Option<String>,
    dry_run: bool,
    swb: &SwbCodec,
) -> Result<()> {
    if !path.exists() {
        bail!("Path does not exist: {}", path.display())
    }

    if path.is_file() {
        let doc = Document::open(path).await?;
        doc.compile(CommandWait::Yes).await?;

        let theme = doc.config().await?.theme;
        let node = &*doc.root_read().await;

        let options = EncodeOptions {
            theme,
            ..Default::default()
        };

        publish_node(node, options, key, dry_run, swb).await
    } else {
        bail!("Publishing of directories is not currently supported")
    }
}

/// Publish a single node
pub async fn publish_node(
    node: &Node,
    options: EncodeOptions,
    key: &Option<String>,
    dry_run: bool,
    swb: &SwbCodec,
) -> Result<()> {
    stencila::publish_node(node, options, key, dry_run, swb).await
}
