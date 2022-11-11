use std::{collections::HashMap, path::Path, sync::Arc};

use common::{
    eyre::Result,
    tokio::sync::{mpsc::UnboundedSender, RwLock},
};
use kernels::KernelSpace;
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
/// - `kernel_space`: The document's kernel space,
///
/// - `patch_sender`: A [`Patch`] channel sender to send patches describing the changes to
pub async fn assemble(
    path: &Path,
    root: &Arc<RwLock<Node>>,
    kernel_space: &Arc<RwLock<KernelSpace>>,
    patch_sender: &UnboundedSender<PatchRequest>,
) -> Result<AddressMap> {
    let kernel_space = kernel_space.read().await;

    let mut address = Address::default();
    let mut context = AssembleContext {
        path: path.into(),
        address_map: AddressMap::default(),
        ids: HashMap::default(),
        kernel_space: &*kernel_space,
        patches: Vec::default(),
    };
    root.write()
        .await
        .assemble(&mut address, &mut context)
        .await?;

    send_patches(patch_sender, context.patches, When::Never);

    Ok(context.address_map)
}
