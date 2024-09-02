use std::path::Path;

use common::eyre::{bail, Result};
use schema::Node;

pub mod cli;
mod stencila;

/// Publish a path (file or directory)
pub async fn publish_path(path: &Path, key: &Option<String>) -> Result<()> {
    if !path.exists() {
        bail!("Path does not exist: {}", path.display())
    }

    if path.is_file() {
        let node = codecs::from_path(path, None).await?;
        publish_node(&node, key).await
    } else {
        bail!("Publishing of directories is not currently supported")
    }
}

/// Publish a single node
pub async fn publish_node(node: &Node, key: &Option<String>) -> Result<()> {
    stencila::publish_node(node, key).await
}
