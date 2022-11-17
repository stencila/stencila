use common::{
    eyre::{eyre, Result},
    tokio::sync::mpsc::UnboundedSender,
    tracing,
};
use graph_triples::Resource;
use node_patch::Patch;
use node_pointer::find;
use stencila_schema::Node;

use crate::messages::{PatchRequest, RequestId, When};

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
            vec![RequestId::new()],
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

/// Sends multiple [`Patch`]es using a channel sender (combining them into a single patch, if
/// possible, before sending)
pub(crate) fn send_patches(
    patch_sender: &UnboundedSender<PatchRequest>,
    patches: Vec<Patch>,
    compile: When,
) {
    if patches.iter().any(|patch| patch.target.is_some()) {
        for patch in patches {
            send_patch(patch_sender, patch, compile)
        }
    } else {
        let patch = Patch::from_patches(patches);
        send_patch(patch_sender, patch, compile)
    }
}
