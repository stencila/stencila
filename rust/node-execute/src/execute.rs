use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    sync::Arc,
};

use eyre::{Report, Result};
use futures::stream::{FuturesUnordered, StreamExt};
use graph::Plan;
use graph_triples::{Resource, ResourceInfo};
use kernels::{KernelSelector, KernelSpace};
use node_address::{Address, AddressMap};
use node_patch::{diff, mutate, Patch};
use stencila_schema::{CodeChunk, CodeExecutableExecuteStatus, CodeExpression, Node};
use tokio::sync::{
    mpsc::{Receiver, UnboundedSender},
    oneshot, RwLock,
};

use crate::{
    utils::{resource_to_node, send_patch, send_patches},
    CancelRequest, Executable, PatchRequest,
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
/// - `patch_request_sender`: A [`PatchRequest`] channel sender to send patches describing the changes to
///                   executed nodes
///
/// - `cancel_request_receiver`: A [`CancelRequest`] channel receiver to request cancellation of one or more
///                   steps in the plan
///
/// - `kernel_space`: The [`KernelSpace`] within which to execute the plan
///
pub async fn execute(
    plan: &Plan,
    root: &Arc<RwLock<Node>>,
    address_map: &Arc<RwLock<AddressMap>>,
    patch_request_sender: &UnboundedSender<PatchRequest>,
    cancel_request_receiver: &mut Receiver<CancelRequest>,
    kernel_space: Option<Arc<KernelSpace>>,
) -> Result<()> {
    let kernel_space = kernel_space.unwrap_or_default();

    // Drain the cancellation channel in case there are any requests inadvertantly
    // sent by a client for a previous execute request.
    while let Ok(..) = cancel_request_receiver.try_recv() {}

    // Obtain locks
    let root_guard = root.read().await;
    let address_map_guard = address_map.read().await;

    // Get a snapshot of all nodes involved in the plan at the start
    let mut node_infos: BTreeMap<Resource, NodeInfo> = plan
        .stages
        .iter()
        .enumerate()
        .flat_map(|(stage_index, stage)| {
            let root_guard = &root_guard;
            let address_map_guard = &address_map_guard;
            stage
                .steps
                .iter()
                .enumerate()
                .filter_map(move |(step_index, step)| {
                    let resource_info = step.resource_info.clone();
                    let resource = &resource_info.resource;
                    match resource_to_node(resource, root_guard, address_map_guard) {
                        Ok((node, node_id, node_address)) => Some((
                            resource.clone(),
                            NodeInfo::new(
                                stage_index,
                                step_index,
                                resource_info,
                                node_id,
                                node_address,
                                node,
                            ),
                        )),
                        Err(error) => {
                            tracing::warn!("While executing plan: {}", error);
                            None
                        }
                    }
                })
        })
        .collect();

    // Release locks
    drop(root_guard);
    drop(address_map_guard);

    // Set the `execute_status` of all nodes in stages other thatn the first
    // to `Scheduled` or `ScheduledPreviouslyFailed` and send the resulting patch.
    // Do not do this for first stage as an optimization to avoid unecessary patches
    // (they will go directly to `Running` or `RunningPreviouslyFailed`)
    send_patches(
        patch_request_sender,
        node_infos
            .values_mut()
            .filter_map(|node_info| {
                if node_info.stage_index != 0 {
                    Some(node_info.set_execute_status_scheduled())
                } else {
                    None
                }
            })
            .collect(),
        true,
    );

    // For each stage in plan...
    let stage_count = plan.stages.len();
    let mut cancelled = Vec::new();
    let mut dependencies_failed = false;
    for (stage_index, stage) in plan.stages.iter().enumerate() {
        // Before running the steps in this stage, check that all their dependencies have succeeded
        // and stop if they have not. Collects to a `BTreeSet` to generate unique set (some steps in
        // the stage may have shared dependencies)
        dependencies_failed = stage
            .steps
            .iter()
            .flat_map(|step| step.resource_info.dependencies.iter().flatten())
            .collect::<BTreeSet<&Resource>>()
            .iter()
            .filter_map(|dependency| node_infos.get(dependency))
            .any(|node_info| {
                tracing::trace!(
                    "Status of dependency of stage {}/{} `{}`: {:?}",
                    stage_index + 1,
                    stage_count,
                    node_info.node_id,
                    node_info.get_execute_status()
                );
                matches!(
                    node_info.get_execute_status(),
                    None | Some(CodeExecutableExecuteStatus::Failed)
                        | Some(CodeExecutableExecuteStatus::Cancelled)
                )
            });
        if dependencies_failed {
            tracing::debug!(
                "Stopping before stage {}/{}: some dependencies failed or were cancelled",
                stage_index + 1,
                stage_count
            );
            break;
        }

        tracing::debug!("Starting stage {}/{}", stage_index + 1, stage_count);

        // Before creating tasks for each step in this stage, check for any cancellation requests
        cancelled.append(&mut collect_cancelled_nodes(
            &mut node_infos,
            cancel_request_receiver,
        ));

        // Create a kernel task for each step in this stage
        let step_count = stage.steps.len();
        let mut patches = Vec::with_capacity(step_count);
        let mut cancellers = HashMap::new();
        let mut futures = Vec::new();
        for (step_index, step) in stage.steps.iter().enumerate() {
            // Get the node info for the step
            let mut node_info = node_infos
                .get(&step.resource_info.resource)
                .cloned()
                .expect("Node info for resource should be available");
            let node_id = node_info.node_id.clone();

            // Has the step been cancelled?
            if cancelled.contains(&node_id) {
                tracing::trace!(
                    "Step for node `{}` was cancelled before it was run",
                    node_id
                );
                // Send a patch to revert `execute_status` to previous status
                // (the `Cancelled` state is reserved for nodes that have started and are cancelled)
                patches.push(node_info.restore_previous_execute_status());
                continue;
            }

            // Set the `execute_status` of the node to `Running` or `RunningPreviouslyFailed`
            // and send the resulting patch
            patches.push(node_info.set_execute_status_running());

            // Create a channel to send cancel requests to task
            let (cancel_sender, cancel_receiver) = oneshot::channel::<()>();

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
                let mut executed = node_info.node.clone();

                // Start execution of the node
                let task_info = match executed
                    .execute_begin(&kernel_space, &kernel_selector, &resource_info, is_fork)
                    .await
                {
                    Ok(task_info) => task_info,
                    Err(error) => {
                        tracing::error!(
                            "While beginning task for step {}/{}: {}",
                            step_index + 1,
                            step_count,
                            error
                        );
                        None
                    }
                };

                // If a `TaskInfo` was returned, the execution is async, can potentially be
                // cancelled, and need to be waited for...
                if let Some(mut task_info) = task_info {
                    // Hook the `cancel_receiver` to the task's `canceller`, if it has one
                    // (i.e. if it is cancellable)
                    if let Some(task_cancel_sender) = task_info.task.canceller.clone() {
                        tracing::trace!("Task is `{}` cancellable", task_info.task.id);
                        let task_id = task_info.task.id.clone();
                        tokio::spawn(async move {
                            if let Ok(..) = cancel_receiver.await {
                                tracing::trace!(
                                    "Attempting to cancel task `{}` in stage {}/{}",
                                    task_id,
                                    stage_index + 1,
                                    stage_count
                                );
                                if let Err(error) = task_cancel_sender.send(()).await {
                                    tracing::error!(
                                        "While attempting to cancel task `{}`: {}",
                                        task_id,
                                        error
                                    );
                                }
                            }
                        });
                    } else {
                        tracing::trace!("Task `{}` is not cancellable", task_info.task.id);
                    };

                    // Wait for the task to finish (or be cancelled and update the executed node when it has
                    let task_result = task_info.result().await?;
                    executed.execute_end(task_info, task_result)?;
                }

                // Update the resource to indicate that the resource was executed
                let execute_status = match &executed {
                    Node::CodeChunk(CodeChunk { execute_status, .. })
                    | Node::CodeExpression(CodeExpression { execute_status, .. }) => {
                        execute_status.clone()
                    }
                    _ => None,
                };
                resource_info.did_execute(execute_status);

                // Generate a patch for the differences resulting from execution
                let mut patch = diff(&node_info.node, &executed);
                patch.address = Some(node_info.node_address.clone());
                patch.target = Some(node_info.node_id.clone());

                // Having generated the patch, update the node_info.node (which may be used
                // for assesing execution status etc)
                node_info.node = executed;

                Ok::<_, Report>((step_index, resource_info, node_info, patch))
            };
            cancellers.insert(node_id, cancel_sender);
            futures.push(future);
        }

        // Send patches for updated execution status
        send_patches(patch_request_sender, patches, true);

        if futures.is_empty() {
            tracing::debug!(
                "Skipping stage {}/{}, all steps cancelled",
                stage_index + 1,
                stage_count
            );
            continue;
        }

        // Spawn all task futures
        // TODO: Replace `FuturesUnordered` with `TaskSet`. See https://news.ycombinator.com/item?id=29912386
        let mut futures_unordered = futures
            .into_iter()
            .map(tokio::spawn)
            .collect::<FuturesUnordered<_>>();

        // Wait for both execution results and any cancellation requests and act accordingly
        loop {
            tokio::select! {
                // Handle tasks that have finished
                result = futures_unordered.next() => {
                    let result = match result {
                        Some(result) => result,
                        // If next() is none, we've reached the end of the tasks so break
                        None => break
                    };
                    let result = match result {
                        Ok(result) => match result {
                            Ok(result) => Some(result),
                            Err(error) => {
                                tracing::error!("While executing a task: {}", error);
                                None
                            }
                        },
                        Err(error) => {
                            tracing::error!("While attempting to join task: {}", error);
                            None
                        }
                    };

                    if let Some((step_index, resource_info, mut node_info, patch)) = result {
                        tracing::debug!(
                            "Finished step {}/{} of stage {}/{}",
                            step_index + 1,
                            step_count,
                            stage_index + 1,
                            stage_count
                        );

                        // Check if step result should be ignored and node not patched
                        if cancelled.contains(&node_info.node_id) {
                            tracing::trace!(
                                "Step for node `{}` was cancelled so result ignored",
                                node_info.node_id
                            );
                            // Send patch to indicate that the node was cancelled i.e. side effects
                            // may have occurred but node will not be patched
                            send_patch(
                                patch_request_sender,
                                node_info.set_execute_status_cancelled(),
                                true
                            );
                        } else {
                            // Send the patch reflecting the changed state of the executed node
                            send_patch(patch_request_sender, patch, true);
                        }

                        // Update the node_info record used elsewhere in this function (mainly for the new execution status of nodes)
                        node_infos
                            .entry(resource_info.resource.clone())
                            .and_modify(|current_node_info| *current_node_info = node_info);
                    }
                }

                // Handle cancellation requests
                Some(request) = cancel_request_receiver.recv() => {
                    if let Some(node_id) = request.start {
                        // Try to find matching cancel channel sender
                        match cancellers.remove(&node_id) {
                            Some(cancel_task_sender) => {
                                tracing::debug!("Cancelling node `{}` in stage {}/{}",
                                    node_id,
                                    stage_index + 1,
                                    stage_count
                                );
                                if let Err(..) = cancel_task_sender.send(()) {
                                    tracing::error!(
                                        "While attempting to cancel node `{}` in stage {}/{}: channel closed",
                                        node_id,
                                        stage_index + 1,
                                        stage_count
                                    );
                                }
                            },
                            None => {
                                tracing::debug!(
                                    "No canceller for node `{}` in stage {}/{}: already cancelled?",
                                    node_id,
                                    stage_index + 1,
                                    stage_count
                                );
                            }
                        }
                        // Add to list of cancelled nodes
                        cancelled.push(node_id);
                    }
                }
            }
        }

        tracing::debug!("Finished stage {}/{}", stage_index + 1, stage_count);
    }

    // For nodes that were scheduled but never got to run because dependencies did not succeed,
    // restore their previous execution status
    if dependencies_failed {
        send_patches(
            patch_request_sender,
            node_infos
                .values_mut()
                .map(|node_info| node_info.restore_previous_execute_status())
                .collect(),
            true,
        );
    }

    Ok(())
}

/// A private internal struct to keep track of details of each node in the
/// execution plan during its execution
#[derive(Clone)]
struct NodeInfo {
    // The index of the stage of the plan the node is in
    stage_index: usize,

    // The index of the step in the stage associated with the node
    step_index: usize,

    /// The associated [`ResourceInfo`]
    resource_info: ResourceInfo,

    /// The id of the node
    node_id: String,

    /// The address of the node
    node_address: Address,

    /// A copy of the node
    ///
    /// We take a copy of the node initially at the start of [`execute`] and
    /// then and send pathces for it to update status and execution results.
    node: Node,

    /// The execution state of the node prior to [`execute`]
    previous_execute_status: Option<CodeExecutableExecuteStatus>,
}

impl NodeInfo {
    fn new(
        stage_index: usize,
        step_index: usize,
        resource_info: ResourceInfo,
        node_id: String,
        node_address: Address,
        node: Node,
    ) -> Self {
        let mut node_info = Self {
            stage_index,
            step_index,
            resource_info,
            node_id,
            node_address,
            node,
            previous_execute_status: None,
        };
        node_info.previous_execute_status = node_info.get_execute_status();
        node_info
    }

    fn get_execute_status(&self) -> Option<CodeExecutableExecuteStatus> {
        match &self.node {
            Node::CodeChunk(CodeChunk { execute_status, .. })
            | Node::CodeExpression(CodeExpression { execute_status, .. }) => execute_status.clone(),
            // At present, assumes the execution of parameters always succeeds
            Node::Parameter(..) => Some(CodeExecutableExecuteStatus::Succeeded),
            _ => None,
        }
    }

    fn set_execute_status_scheduled(&mut self) -> Patch {
        mutate(
            &mut self.node,
            Some(self.node_id.to_string()),
            Some(self.node_address.clone()),
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

    fn set_execute_status_running(&mut self) -> Patch {
        mutate(
            &mut self.node,
            Some(self.node_id.to_string()),
            Some(self.node_address.clone()),
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

    fn set_execute_status_cancelled(&mut self) -> Patch {
        mutate(
            &mut self.node,
            Some(self.node_id.to_string()),
            Some(self.node_address.clone()),
            &|node: &mut Node| match node {
                Node::CodeChunk(CodeChunk { execute_status, .. })
                | Node::CodeExpression(CodeExpression { execute_status, .. }) => {
                    *execute_status = Some(CodeExecutableExecuteStatus::Cancelled);
                }
                _ => {}
            },
        )
    }

    fn restore_previous_execute_status(&mut self) -> Patch {
        mutate(
            &mut self.node,
            Some(self.node_id.to_string()),
            Some(self.node_address.clone()),
            &|node: &mut Node| match node {
                Node::CodeChunk(CodeChunk { execute_status, .. })
                | Node::CodeExpression(CodeExpression { execute_status, .. }) => {
                    if matches!(
                        execute_status,
                        Some(CodeExecutableExecuteStatus::Scheduled)
                            | Some(CodeExecutableExecuteStatus::ScheduledPreviouslyFailed)
                            | Some(CodeExecutableExecuteStatus::Running)
                            | Some(CodeExecutableExecuteStatus::RunningPreviouslyFailed)
                    ) {
                        *execute_status = self.previous_execute_status.clone();
                    }
                }
                _ => {}
            },
        )
    }
}

fn collect_cancelled_nodes(
    node_infos: &mut BTreeMap<Resource, NodeInfo>,
    cancel_request_receiver: &mut Receiver<CancelRequest>,
) -> Vec<String> {
    let mut cancelled = Vec::new();
    while let Ok(request) = cancel_request_receiver.try_recv() {
        if let Some(start) = request.start {
            // Cancel execution a specific node and optionally all its downsteams
            cancelled.push(start)
        } else {
            // Cancel execution of all nodes
            cancelled.append(
                &mut node_infos
                    .values()
                    .map(|node_info| node_info.node_id.clone())
                    .collect(),
            );
        }
    }
    cancelled
}
