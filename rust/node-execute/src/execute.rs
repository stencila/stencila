use std::sync::Arc;

use eyre::{bail, eyre, Result};
use futures::stream::{FuturesUnordered, StreamExt};
use graph::Plan;
use graph_triples::ResourceInfo;
use kernels::{KernelSelector, KernelSpace};
use node_address::AddressMap;
use node_patch::{diff, Patch};
use node_pointer::resolve;
use stencila_schema::{CodeChunk, CodeExecutableExecuteStatus, CodeExpression, Node};
use tokio::sync::{
    mpsc::{Sender, UnboundedSender},
    RwLock,
};

use crate::Executable;

/// Execute a [`Plan`] on a [`Node`]
///
/// Uses a a `RwLock` for `root` and `address_map` so that locks can be held for as short as
/// time as possible (i.e. not while waiting for execution of tasks, which is what would
/// happen if held by the caller).
///
/// # Arguments
///
/// - `plan`: The plan to be executed
///
/// - `root`: The root node to execute the plan on
///
/// - `address_map`: The [`AddressMap`] map for the `root` node (used to locate code nodes
///                  included in the plan within the `root` node)
///
/// - `resource_info_sender`: A [`ResourceInfo`] channel sender to update the graph on the
///                  execution status of resources
///
/// - `patch_sender`: A [`Patch`] channel sender to send patches describing the changes to
///                   executed nodes
///
/// - `kernel_space`: The `KernelSpace` within which to execute the plan
///
pub async fn execute(
    plan: &Plan,
    root: &Arc<RwLock<Node>>,
    address_map: &Arc<RwLock<AddressMap>>,
    resource_info_sender: &Sender<ResourceInfo>,
    patch_sender: &UnboundedSender<Patch>,
    kernel_space: Option<Arc<KernelSpace>>,
) -> Result<()> {
    let kernel_space = kernel_space.unwrap_or_default();

    // For each stage in plan...
    let stage_count = plan.stages.len();
    for (stage_index, stage) in plan.stages.iter().enumerate() {
        tracing::debug!("Starting stage {}/{}", stage_index + 1, stage_count);

        // Create a kernel task for each step in this stage
        let step_count = stage.steps.len();
        let mut futures = Vec::with_capacity(step_count);
        // Get locks
        let root = root.read().await;
        let address_map = address_map.read().await;
        for (step_index, step) in stage.steps.iter().enumerate() {
            // Get a pointer to the step's node from the root node
            let node_id = step
                .resource_info
                .resource
                .node_id()
                .ok_or_else(|| eyre!("Expected to get code id for resource"))?;
            let node_address = address_map.get(&node_id).cloned();
            let node_id = Some(node_id.clone());
            let pointer = resolve(&*root, node_address.clone(), node_id.clone())?;

            // Create a copy of the node that can be moved to the async task and create clones
            // of other variables needed to execute the task
            let before = pointer.to_node()?;
            let kernel_space = kernel_space.clone();
            let kernel_selector = KernelSelector::new(step.kernel_name.clone(), None, None);
            let mut resource_info = step.resource_info.clone();
            let is_fork = step.is_fork;

            // Create a future for the task that will be spawned later
            let future = async move {
                tracing::debug!(
                    "Starting step {}/{} of stage {}/{}",
                    step_index + 1,
                    step_count,
                    stage_index + 1,
                    stage_count
                );

                // Create a mutable copy of the code and execute it in the kernel space
                let mut after = before.clone();
                match after
                    .execute(&kernel_space, &kernel_selector, &resource_info, is_fork)
                    .await
                {
                    Ok(_) => {
                        // Update the resource to indicate that the resource was executed
                        let succeeded = match &after {
                            Node::CodeChunk(CodeChunk { execute_status, .. })
                            | Node::CodeExpression(CodeExpression { execute_status, .. }) => {
                                execute_status.as_ref().and_then(|status| match status {
                                    CodeExecutableExecuteStatus::Succeeded => Some(true),
                                    CodeExecutableExecuteStatus::Failed => Some(false),
                                    _ => None,
                                })
                            }
                            _ => None,
                        };
                        resource_info.did_execute(succeeded);

                        // Generate a patch for the differences resulting from execution
                        let mut patch = diff(&before, &after);
                        patch.address = node_address;
                        patch.target = node_id;

                        Ok((step_index, resource_info, patch))
                    }
                    Err(error) => bail!(error),
                }
            };
            futures.push(future);
        }
        // Release locks
        drop(root);
        drop(address_map);

        // Spawn all tasks in the stage and wait for each to finish, sending on the resultant `Patch`
        // for application and publishing (if it is not empty)
        // TODO: Replace `FuturesUnordered` with `TaskSet`. See https://news.ycombinator.com/item?id=29912386
        let mut results = futures
            .into_iter()
            .map(tokio::spawn)
            .collect::<FuturesUnordered<_>>();
        while let Some(result) = results.next().await {
            let (step_index, resource_info, patch) = result??;

            tracing::debug!(
                "Finished step {}/{} of stage {}/{}",
                step_index + 1,
                step_count,
                stage_index + 1,
                stage_count
            );

            if let Err(error) = resource_info_sender.send(resource_info).await {
                tracing::debug!(
                    "When sending resource info for step {} of stage {}: {}",
                    step_index + 1,
                    stage_index + 1,
                    error
                );
            }

            if !patch.is_empty() {
                if let Err(error) = patch_sender.send(patch) {
                    tracing::debug!(
                        "When sending patch for step {} of stage {}: {}",
                        step_index + 1,
                        stage_index + 1,
                        error
                    );
                }
            }
        }

        tracing::debug!("Finished stage {}/{}", stage_index + 1, stage_count);
    }

    Ok(())
}
