use std::{collections::BTreeMap, sync::Arc};

use eyre::{bail, Result};
use futures::stream::{FuturesUnordered, StreamExt};
use graph::{Graph, Plan};
use graph_triples::Resource;
use kernels::{KernelSelector, KernelSpace};
use node_address::{Address, AddressMap};
use node_patch::{diff, mutate, Patch};
use stencila_schema::{CodeChunk, CodeExecutableExecuteStatus, CodeExpression, Node};
use tokio::sync::{mpsc::UnboundedSender, RwLock};

use crate::{
    compile_no_walk,
    utils::{resource_to_node, send_patches},
    Executable,
};

/// Execute a [`Plan`] on a [`Node`]
///
/// Uses a `RwLock` for `root` and `address_map` so that read locks can be held for as short as
/// time as possible (i.e. not while waiting for execution of tasks, which is what would
/// happen if held by the caller).
///
/// # Arguments
///
/// - `plan`: The plan to be executed
///
/// - `root`: The root node to execute the plan on (takes a read lock)
///
/// - `address_map`: The [`AddressMap`] map for the `root` node (used to locate code nodes
///                  included in the plan within the `root` node; takes a read lock)
///
/// - `patch_sender`: A [`Patch`] channel sender to send patches describing the changes to
///                   executed nodes
///
/// - `kernel_space`: The [`KernelSpace`] within which to execute the plan
///
pub async fn execute(
    plan: &Plan,
    root: &Arc<RwLock<Node>>,
    address_map: &Arc<RwLock<AddressMap>>,
    graph: &Arc<RwLock<Graph>>,
    patch_sender: &UnboundedSender<Patch>,
    kernel_space: Option<Arc<KernelSpace>>,
) -> Result<()> {
    let kernel_space = kernel_space.unwrap_or_default();

    // Obtain locks
    let root_guard = root.read().await;
    let address_map_guard = address_map.read().await;

    // Get a snapshot of all nodes involved in the plan at the start
    // Also get their `execute_status` so it can be restored at the end if needs be.
    let mut nodes: BTreeMap<Resource, _> = plan
        .stages
        .iter()
        .flat_map(|stage| stage.steps.iter())
        .filter_map(|step| {
            let resource_info = step.resource_info.clone();
            let resource = &resource_info.resource;
            match resource_to_node(resource, &root_guard, &address_map_guard) {
                Ok((node, node_id, node_address)) => {
                    let execution_status = get_execute_status(&node);
                    Some((
                        resource.clone(),
                        (resource_info, node_id, node_address, node, execution_status),
                    ))
                }
                Err(error) => {
                    tracing::warn!("While executing plan: {}", error);
                    None
                }
            }
        })
        .collect();

    // Release locks
    drop(root_guard);
    drop(address_map_guard);

    // Set the `execute_status` of all nodes to `Scheduled` or `ScheduledPreviouslyFailed`
    // and send the resulting patch
    send_patches(
        patch_sender,
        nodes
            .values_mut()
            .map(|(_, node_id, node_address, node, ..)| {
                set_execute_status_scheduled(node_id, node_address, node)
            })
            .collect(),
    );

    // For each stage in plan...
    let stage_count = plan.stages.len();
    let mut dependencies_failed = false;
    for (stage_index, stage) in plan.stages.iter().enumerate() {
        tracing::debug!("Starting stage {}/{}", stage_index + 1, stage_count);

        // Before running the steps in this stage, check that all their dependencies have succeeded
        // and stop if they have not
        dependencies_failed = stage
            .steps
            .iter()
            .flat_map(|step| step.resource_info.dependencies.iter().flatten())
            .filter_map(|dependency| nodes.get(dependency))
            .map(|tuple| get_execute_status(&tuple.3))
            .any(|status| !matches!(status, Some(CodeExecutableExecuteStatus::Succeeded)));
        if dependencies_failed {
            break;
        }

        // Create a kernel task for each step in this stage
        let step_count = stage.steps.len();
        let mut patches = Vec::with_capacity(step_count);
        let mut futures = Vec::with_capacity(step_count);
        for (step_index, step) in stage.steps.iter().enumerate() {
            // Get the node
            let (_, node_id, node_address, mut node, ..) = nodes
                .get(&step.resource_info.resource)
                .expect("Node for resource should be in nodes")
                .clone();

            // Set the `execute_status` of the node to `Running` or `RunningPreviouslyFailed`
            // and send the resulting patch
            patches.push(set_execute_status_running(
                &node_id,
                &node_address,
                &mut node,
            ));

            // Create clones of variables needed to execute the task
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

                // Create a mutable draft of the node and execute it in the kernel space
                let mut draft = node.clone();
                match draft
                    .execute(&kernel_space, &kernel_selector, &resource_info, is_fork)
                    .await
                {
                    Ok(_) => {
                        // Update the resource to indicate that the resource was executed
                        let execute_status = match &draft {
                            Node::CodeChunk(CodeChunk { execute_status, .. })
                            | Node::CodeExpression(CodeExpression { execute_status, .. }) => {
                                execute_status.clone()
                            }
                            _ => None,
                        };
                        resource_info.did_execute(execute_status);

                        // Generate a patch for the differences resulting from execution
                        let mut patch = diff(&node, &draft);
                        patch.address = Some(node_address.clone());
                        patch.target = Some(node_id.clone());

                        Ok((step_index, resource_info, draft, patch))
                    }
                    Err(error) => bail!(error),
                }
            };
            futures.push(future);
        }
        send_patches(patch_sender, patches);

        // Spawn all tasks in the stage and wait for each to finish, sending on the resultant `Patch`
        // for application and publishing (if it is not empty)
        // TODO: Replace `FuturesUnordered` with `TaskSet`. See https://news.ycombinator.com/item?id=29912386
        let mut results = futures
            .into_iter()
            .map(tokio::spawn)
            .collect::<FuturesUnordered<_>>();
        while let Some(result) = results.next().await {
            let (step_index, resource_info, node, patch) = result??;

            tracing::debug!(
                "Finished step {}/{} of stage {}/{}",
                step_index + 1,
                step_count,
                stage_index + 1,
                stage_count
            );

            // Send the patch reflecting the changed state of the executed node
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

            nodes
                .entry(resource_info.resource.clone())
                .and_modify(|info| info.3 = node);

            // Update the graph with new resource info
            graph.write().await.update_resource_info(resource_info)?;

            // Then do a recompile, using the graph, so that node properties such as
            // `code_dependencies` and `code_dependents` get updated with the new execution status
            // of nodes.
            compile_no_walk(root, address_map, graph, patch_sender).await?;
        }

        tracing::debug!("Finished stage {}/{}", stage_index + 1, stage_count);
    }

    // For nodes that were scheduled but never got to run because dependencies did not succeed,
    // restore their previous execution status
    if dependencies_failed {
        send_patches(
            patch_sender,
            nodes
                .values_mut()
                .map(|(_, node_id, node_address, node, execute_status)| {
                    restore_previous_execute_status(node_id, node_address, node, execute_status)
                })
                .collect(),
        );
    }

    Ok(())
}

fn get_execute_status(node: &Node) -> Option<CodeExecutableExecuteStatus> {
    match node {
        Node::CodeChunk(CodeChunk { execute_status, .. })
        | Node::CodeExpression(CodeExpression { execute_status, .. }) => execute_status.clone(),
        _ => None,
    }
}

fn set_execute_status_scheduled(node_id: &str, node_address: &Address, node: &mut Node) -> Patch {
    mutate(
        node,
        Some(node_id.to_string()),
        Some(node_address.clone()),
        &|node: &mut Node| match node {
            Node::CodeChunk(CodeChunk { execute_status, .. })
            | Node::CodeExpression(CodeExpression { execute_status, .. }) => {
                *execute_status = Some(match execute_status {
                    Some(CodeExecutableExecuteStatus::Failed) => {
                        CodeExecutableExecuteStatus::ScheduledPreviouslyFailed
                    }
                    _ => CodeExecutableExecuteStatus::Scheduled,
                });
            }
            _ => {}
        },
    )
}

fn set_execute_status_running(node_id: &str, node_address: &Address, node: &mut Node) -> Patch {
    mutate(
        node,
        Some(node_id.to_string()),
        Some(node_address.clone()),
        &|node: &mut Node| match node {
            Node::CodeChunk(CodeChunk { execute_status, .. })
            | Node::CodeExpression(CodeExpression { execute_status, .. }) => {
                *execute_status = Some(match execute_status {
                    Some(CodeExecutableExecuteStatus::Failed)
                    | Some(CodeExecutableExecuteStatus::ScheduledPreviouslyFailed) => {
                        CodeExecutableExecuteStatus::RunningPreviouslyFailed
                    }
                    _ => CodeExecutableExecuteStatus::Running,
                });
            }
            _ => {}
        },
    )
}

fn restore_previous_execute_status(
    node_id: &str,
    node_address: &Address,
    node: &mut Node,
    previous_execute_status: &Option<CodeExecutableExecuteStatus>,
) -> Patch {
    mutate(
        node,
        Some(node_id.to_string()),
        Some(node_address.clone()),
        &|node: &mut Node| match node {
            Node::CodeChunk(CodeChunk { execute_status, .. })
            | Node::CodeExpression(CodeExpression { execute_status, .. }) => {
                if matches!(
                    execute_status,
                    Some(CodeExecutableExecuteStatus::Scheduled)
                        | Some(CodeExecutableExecuteStatus::ScheduledPreviouslyFailed)
                ) {
                    *execute_status = previous_execute_status.clone();
                }
            }
            _ => {}
        },
    )
}
