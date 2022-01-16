use eyre::{eyre, Result};
use graph_triples::Resource;
use node_address::{Address, AddressMap};
use node_patch::Patch;
use node_pointer::resolve;
use stencila_schema::Node;
use tokio::sync::mpsc::UnboundedSender;

/// Get the [`Node`] corresponding to a [`Resource`]
///
/// # Arguments
///
/// - `resource`: The [`Resource::Node`] or [`Resource::Code`] that refers to a node in `root`
///
/// - `root`: The root [`Node`] that contains the referred to node
///
/// - `address_map`: The [`AddressMap`] for `root` used to [`resolve`] the node
pub(crate) fn resource_to_node(
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

/// Sends a [`Patch`] using a channel sender (if the patch is not empty)
pub(crate) fn send_patch(patch_sender: &UnboundedSender<Patch>, patch: Patch) {
    if !patch.is_empty() {
        if let Err(error) = patch_sender.send(patch) {
            tracing::debug!("When sending patch: {}", error);
        }
    }
}

/// Sends multiple [`Patch`]es using a channel sender (combining them into a single patch before sending)
pub(crate) fn send_patches(patch_sender: &UnboundedSender<Patch>, patches: Vec<Patch>) {
    let patch = Patch::from_patches(patches);
    send_patch(patch_sender, patch)
}
