use common::{
    eyre::{eyre, Result},
    tokio::sync::mpsc::UnboundedSender,
    tracing,
};
use graph_triples::Resource;
use node_address::{Address, AddressMap};
use node_patch::Patch;
use node_pointer::resolve;
use stencila_schema::Node;

use crate::{PatchRequest, When};

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
        .get(node_id)
        .ok_or_else(|| eyre!("Expected to have address for node `{}`", node_id))?
        .clone();

    let pointer = resolve(
        &*root,
        Some(node_address.clone()),
        Some(node_id.to_string()),
    )?;
    let node = pointer.to_node()?;

    Ok((node, node_id.to_string(), node_address))
}

/// Sends a [`Patch`] using a channel sender (if the patch is not empty)
///
/// Use `compile == true` in `execute()` function but not in `compile()` function to avoid
/// infinite loops.
pub(crate) fn send_patch(
    patch_sender: &UnboundedSender<PatchRequest>,
    patch: Patch,
    compile: When,
) {
    if !patch.is_empty() {
        tracing::trace!(
            "Sending patch request with `{}` operations",
            patch.ops.len()
        );
        // Note: this patch requests do not execute or write after the patch is applied
        if let Err(..) = patch_sender.send(PatchRequest::new(
            patch,
            When::Now,
            compile,
            When::Never,
            When::Never,
        )) {
            tracing::debug!("When sending patch: receiver dropped");
        }
    }
}

/// Sends multiple [`Patch`]es using a channel sender (combining them into a single patch before sending)
pub(crate) fn send_patches(
    patch_sender: &UnboundedSender<PatchRequest>,
    patches: Vec<Patch>,
    compile: When,
) {
    let patch = Patch::from_patches(patches);
    send_patch(patch_sender, patch, compile)
}
