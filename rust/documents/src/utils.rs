use common::eyre::{eyre, Result};
use graph_triples::Resource;
use node_pointer::find;
use stencila_schema::Node;

/// Get the [`Node`] corresponding to a [`Resource`]
///
/// # Arguments
///
/// - `resource`: The [`Resource::Node`] or [`Resource::Code`] that refers to a node in `root`
///
/// - `root`: The root [`Node`] that contains the referred to node
pub(crate) fn resource_to_node(resource: &Resource, root: &Node) -> Result<(Node, String)> {
    let node_id = resource.node_id().ok_or_else(|| {
        eyre!(
            "Expected to have node id for resource `{}`",
            resource.resource_id()
        )
    })?;

    let pointer = find(root, node_id)?;
    let node = pointer.to_node()?;

    Ok((node, node_id.to_string()))
}
