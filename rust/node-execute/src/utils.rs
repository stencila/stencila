use eyre::{eyre, Result};
use graph_triples::Resource;
use node_address::{Address, AddressMap};
use node_pointer::resolve;
use stencila_schema::Node;

/// Get the [`Node`] corresponding to a [`Resource`]
///
/// # Arguments
///
/// - `resource`: The [`Resource::Node`] or [`Resource::Code`] that refers to a node in `root`
///
/// - `root`: The root [`Node`] that contains the referred to node
///
/// - `address_map`: The [`AddressMap`] for `root` used to [`resolve`] the node
pub fn resource_to_node(
    resource: &Resource,
    root: &Node,
    address_map: &AddressMap,
) -> Result<(Node, String, Address)> {
    let node_id = resource.node_id().ok_or_else(|| {
        eyre!(
            "Expected to have node id for resource `{}`",
            resource.resource_id()
        )
    })?;

    let node_address = address_map
        .get(&node_id)
        .ok_or_else(|| eyre!("Expected to have address for node `{}`", node_id))?
        .clone();

    let pointer = resolve(&*root, Some(node_address.clone()), Some(node_id.clone()))?;
    let node = pointer.to_node()?;

    Ok((node, node_id, node_address))
}
