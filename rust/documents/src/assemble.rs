use std::{path::Path, sync::Arc};

use common::{
    eyre::Result,
    tokio::sync::{mpsc::UnboundedSender, RwLock},
};
use node_address::{Address, AddressMap};
use stencila_schema::Node;

use crate::{
    executable::{AssembleContext, Executable},
    messages::{PatchRequest, When},
    utils::send_patches,
};

/// Assemble a node, usually the `root` node of a document
///
/// Uses a `RwLock` for `root` so that write lock can be held for as short as
/// time as possible and for consistency with the `execute` function.
///
/// # Arguments
///
/// - `path`: The path of the document to be compiled
///
/// - `root`: The root node to be compiled
///
/// - `patch_sender`: A [`Patch`] channel sender to send patches describing the changes to
pub async fn assemble(
    path: &Path,
    root: &Arc<RwLock<Node>>,
    patch_sender: &UnboundedSender<PatchRequest>,
) -> Result<AddressMap> {
    let mut address = Address::default();
    let mut context = AssembleContext {
        path: path.into(),
        ..Default::default()
    };
    root.write()
        .await
        .assemble(&mut address, &mut context)
        .await?;

    send_patches(patch_sender, context.patches, When::Never);

    Ok(context.address_map)
}
