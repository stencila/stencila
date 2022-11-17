use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    sync::Arc,
};

use common::{
    eyre::{bail, Report, Result},
    futures::stream::{FuturesUnordered, StreamExt},
    tokio::{
        self,
        sync::{
            mpsc::{Receiver, UnboundedSender},
            oneshot, RwLock,
        },
    },
    tracing,
};
use graph::{Plan, PlanScope};
use graph_triples::{Resource, TagMap};
use kernels::KernelSpace;

use node_patch::{diff, mutate, Patch};
use stencila_schema::{CodeChunk, CodeExpression, Division, ExecuteStatus, Node, Span};

use crate::{
    executable::Executable,
    messages::{CancelRequest, PatchRequest, When},
    utils::{resource_to_node, send_patch, send_patches},
};

/// Execute a [`Plan`] on a [`Node`]
///
/// # Arguments
///
/// - `plan`: The plan to be executed
///
/// - `root`: The root node to execute the plan on (takes a read lock)

/// - `tag_map`: The document's [`TagMap`] of global tags
///
/// - `kernel_space`: The [`KernelSpace`] within which to execute the plan
///
/// - `patch_request_sender`: A [`PatchRequest`] channel sender to send patches describing the
///                   changes to executed nodes
///
/// - `cancel_request_receiver`: A [`CancelRequest`] channel receiver to request cancellation of
///                   one or more tasks in the plan
#[allow(clippy::too_many_arguments)]
pub async fn execute(
    plan: &Plan,
    root: &Arc<RwLock<Node>>,
    tag_map: &Arc<RwLock<TagMap>>,
    kernel_space: &Arc<RwLock<KernelSpace>>,
    patch_request_sender: &UnboundedSender<PatchRequest>,
    cancel_request_receiver: &mut Receiver<CancelRequest>,
) -> Result<()> {
    // Drain the cancellation channel in case there are any requests inadvertantly
    // sent by a client for a previous execute request.
    while let Ok(..) = cancel_request_receiver.try_recv() {}

    // Obtain locks
    let root_guard = root.read().await;

    // Get a snapshot of all nodes involved in the plan at the start
    let mut node_infos: BTreeMap<Resource, NodeInfo> = plan
        .stages
        .iter()
        .enumerate()
        .flat_map(|(stage_index, stage)| {
            let root_guard = &root_guard;
            stage
                .tasks
                .iter()
                .enumerate()
                .filter_map(move |(.., task)| {
                    let resource_info = task.resource_info.clone();
                    let resource = &resource_info.resource;
                    match resource_to_node(resource, root_guard) {
                        Ok((node, node_id)) => {
                            Some((resource.clone(), NodeInfo::new(stage_index, node_id, node)))
                        }
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

    // Set the `execute_status` of all nodes in stages other than the first
    // to `Scheduled` or `ScheduledPreviouslyFailed` and send the resulting patch.
    // Do not do this for first stage as an optimization to avoid unnecessary patches
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
        When::Soon,
    );

    // For each stage in plan...
    let stage_count = plan.stages.len();
    let mut cancelled = Vec::new();
    for (stage_index, stage) in plan.stages.iter().enumerate() {
        // Before running the tasks in this stage, check that all their dependencies have succeeded
        // and stop if they have not. Collects to a `BTreeSet` to generate unique set (some tasks in
        // the stage may have shared dependencies)
        let dependencies_failed = stage
            .tasks
            .iter()
            .flat_map(|task| task.resource_info.dependencies.iter().flatten())
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
                    None | Some(ExecuteStatus::Failed) | Some(ExecuteStatus::Cancelled)
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

        tracing::trace!("Starting stage {}/{}", stage_index + 1, stage_count);

        let task_count = stage.tasks.len();
        let mut patches = Vec::with_capacity(task_count);
        let mut futures = Vec::new();
        let mut cancellers = HashMap::new();

        // Before creating tasks check for any cancellation requests
        let mut cancel_all = false;
        while let Ok(request) = cancel_request_receiver.try_recv() {
            cancel_all = handle_cancel_request(
                request,
                &node_infos,
                &mut cancellers,
                &mut cancelled,
                patch_request_sender,
            );
            if cancel_all {
                break;
            }
        }
        if cancel_all {
            break;
        }

        // Create a kernel task for each task in this stage
        let tags = tag_map.read().await;
        for (task_index, task) in stage.tasks.iter().enumerate() {
            // Get the node info for the task
            let mut node_info = match node_infos.get(&task.resource_info.resource) {
                Some(node_info) => node_info.clone(),
                None => {
                    bail!(
                        "Node info is not available for resource `{}`",
                        &task.resource_info.resource.resource_id()
                    )
                }
            };
            let node_id = node_info.node_id.clone();

            // Has the task been cancelled?
            if cancelled.contains(&node_id) {
                tracing::debug!(
                    "Execution of node `{}` was cancelled before it was started",
                    node_id
                );
                // Send a patch to revert `execute_status` to previous status
                // (the `Cancelled` state is reserved for nodes that have started and are cancelled)
                patches.push(node_info.reset_execute_status());
                continue;
            }

            // Set the `execute_status` of the node to `Running` or `RunningPreviouslyFailed`
            // and send the resulting patch
            patches.push(node_info.set_execute_status_running());

            // Create a channel to send cancel requests to task
            let (cancel_sender, cancel_receiver) = oneshot::channel::<()>();

            // Create clones of variables needed to execute the task
            let kernel_space = kernel_space.clone();
            let mut resource_info = task.resource_info.clone();
            let kernel_selector = task.kernel_selector.clone();
            let is_fork = task.is_fork;

            // Insert the document's global tags into the resource's
            resource_info.tags.insert_globals(&*tags);

            // Create a future for the task that will be spawned later
            let future = async move {
                tracing::trace!(
                    "Starting task {}/{} of stage {}/{}",
                    task_index + 1,
                    task_count,
                    stage_index + 1,
                    stage_count
                );

                // Create a mutable draft of the node and execute it in the kernel space
                let mut executed = node_info.node.clone();

                // Start execution of the node
                let task_info = match executed
                    .execute_begin(
                        &resource_info,
                        &*kernel_space.read().await,
                        &kernel_selector,
                        is_fork,
                    )
                    .await
                {
                    Ok(task_info) => task_info,
                    Err(error) => {
                        tracing::error!(
                            "While beginning task {}/{}: {}",
                            task_index + 1,
                            task_count,
                            error
                        );
                        None
                    }
                };

                // If a `TaskInfo` was returned, the execution is async, can potentially be
                // interrupted, and needs to be waited for...
                if let Some(mut task_info) = task_info {
                    // Hook the `cancel_receiver` to the task's `interrupter`, if it has one
                    // (i.e. if it is interruptable)
                    if let Some(interrupter) = task_info.task.interrupter.clone() {
                        tracing::trace!("Task is `{}` interruptable", task_info.task.id);
                        let task_id = task_info.task.id.clone();
                        tokio::spawn(async move {
                            if let Ok(..) = cancel_receiver.await {
                                tracing::trace!(
                                    "Attempting to interrupt task `{}` in stage {}/{}",
                                    task_id,
                                    stage_index + 1,
                                    stage_count
                                );
                                if let Err(error) = interrupter.send(()).await {
                                    tracing::error!(
                                        "While attempting to cancel task `{}`: {}",
                                        task_id,
                                        error
                                    );
                                }
                            }
                        });
                    } else {
                        tracing::trace!("Task `{}` is not interruptable", task_info.task.id);
                    };

                    // Wait for the task to finish (or be cancelled and update the executed node when it has)
                    let task_result = task_info.result().await?;
                    executed.execute_end(task_info, task_result).await?;
                }

                // Update the resource to indicate that the resource was executed
                let execute_failed = match &executed {
                    Node::CodeChunk(CodeChunk { execute_status, .. })
                    | Node::CodeExpression(CodeExpression { execute_status, .. })
                    | Node::Division(Division { execute_status, .. })
                    | Node::Span(Span { execute_status, .. }) => {
                        matches!(execute_status, Some(ExecuteStatus::Failed))
                    }
                    _ => false,
                };
                resource_info.did_execute(execute_failed);

                // Generate a patch for the differences resulting from execution
                let mut patch = diff(&node_info.node, &executed);
                patch.target = Some(node_info.node_id.clone());

                // Having generated the patch, update the node_info.node (which may be used
                // for assessing execution status etc)
                node_info.node = executed;

                Ok::<_, Report>((task_index, resource_info, node_info, patch))
            };
            cancellers.insert(node_id, cancel_sender);
            futures.push(future);
        }
        drop(tags);

        // Send patches for updated execution status
        send_patches(patch_request_sender, patches, When::Soon);

        if futures.is_empty() {
            tracing::debug!(
                "Skipping stage {}/{}, all tasks cancelled",
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

                    if let Some((task_index, resource_info, mut node_info, patch)) = result {
                        tracing::trace!(
                            "Finished task {}/{} of stage {}/{}",
                            task_index + 1,
                            task_count,
                            stage_index + 1,
                            stage_count
                        );

                        // Check if task result should be ignored and node not patched
                        if cancelled.contains(&node_info.node_id) {
                            tracing::trace!(
                                "Execution of node `{}` was cancelled so result was ignored",
                                node_info.node_id
                            );
                            // Send patch to indicate that the node was cancelled i.e. side effects
                            // may have occurred but node will not be patched
                            send_patch(
                                patch_request_sender,
                                node_info.set_execute_status_cancelled(),
                                When::Soon
                            );
                        } else {
                            // Send the patch reflecting the changed state of the executed node
                            send_patch(patch_request_sender, patch, When::Soon);
                        }

                        // Update the node_info record used elsewhere in this function (mainly for the new execution status of nodes)
                        node_infos
                            .entry(resource_info.resource.clone())
                            .and_modify(|current_node_info| *current_node_info = node_info);
                    }
                }

                // Handle cancellation requests, exiting the loop if the cancellation scope is
                // `All` (i.e the whole plan)
                Some(request) = cancel_request_receiver.recv() => {
                    let all = handle_cancel_request(request, &node_infos, &mut cancellers, &mut cancelled, patch_request_sender);
                    if all {
                        break;
                    }
                }
            }
        }

        tracing::trace!("Finished stage {}/{}", stage_index + 1, stage_count);
    }

    // For nodes that were scheduled but never got to run (e.g. because dependencies did not succeed
    // or the plan was cancelled), or were running but got cancelled, reset execute status
    send_patches(
        patch_request_sender,
        node_infos
            .values_mut()
            .map(|node_info| node_info.reset_execute_status())
            .collect(),
        When::Soon,
    );

    Ok(())
}

/// A private internal struct to keep track of details of each node in the
/// execution plan during its execution
#[derive(Clone)]
struct NodeInfo {
    // The index of the stage of the plan the node is in
    stage_index: usize,

    /// The id of the node
    node_id: String,

    /// A copy of the node
    ///
    /// We take a copy of the node initially at the start of [`execute`] and
    /// then and send patches for it to update status and execution results.
    node: Node,

    /// The execution state of the node prior to [`execute`]
    previous_execute_status: Option<ExecuteStatus>,
}

impl NodeInfo {
    fn new(stage_index: usize, node_id: String, node: Node) -> Self {
        let mut node_info = Self {
            stage_index,
            node_id,
            node,
            previous_execute_status: None,
        };
        node_info.previous_execute_status = node_info.get_execute_status();
        node_info
    }

    fn get_execute_status(&self) -> Option<ExecuteStatus> {
        match &self.node {
            Node::CodeChunk(CodeChunk { execute_status, .. })
            | Node::CodeExpression(CodeExpression { execute_status, .. })
            | Node::Division(Division { execute_status, .. })
            | Node::Span(Span { execute_status, .. }) => execute_status.clone(),
            // At present, assumes the execution of parameters and buttons always succeeds
            Node::Parameter(..) | Node::Button(..) => Some(ExecuteStatus::Succeeded),
            _ => None,
        }
    }

    fn set_execute_status_scheduled(&mut self) -> Patch {
        mutate(
            &mut self.node,
            Some(self.node_id.to_string()),
            None,
            &|node: &mut Node| match node {
                Node::CodeChunk(CodeChunk { execute_status, .. })
                | Node::CodeExpression(CodeExpression { execute_status, .. })
                | Node::Division(Division { execute_status, .. })
                | Node::Span(Span { execute_status, .. }) => {
                    *execute_status = Some(match execute_status {
                        Some(ExecuteStatus::Failed) => ExecuteStatus::ScheduledPreviouslyFailed,
                        _ => ExecuteStatus::Scheduled,
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
            None,
            &|node: &mut Node| match node {
                Node::CodeChunk(CodeChunk { execute_status, .. })
                | Node::CodeExpression(CodeExpression { execute_status, .. })
                | Node::Division(Division { execute_status, .. })
                | Node::Span(Span { execute_status, .. }) => {
                    *execute_status = Some(match execute_status {
                        Some(ExecuteStatus::Failed)
                        | Some(ExecuteStatus::ScheduledPreviouslyFailed) => {
                            ExecuteStatus::RunningPreviouslyFailed
                        }
                        _ => ExecuteStatus::Running,
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
            None,
            &|node: &mut Node| match node {
                Node::CodeChunk(CodeChunk { execute_status, .. })
                | Node::CodeExpression(CodeExpression { execute_status, .. })
                | Node::Division(Division { execute_status, .. })
                | Node::Span(Span { execute_status, .. }) => {
                    *execute_status = Some(ExecuteStatus::Cancelled);
                }
                _ => {}
            },
        )
    }

    fn reset_execute_status(&mut self) -> Patch {
        mutate(
            &mut self.node,
            Some(self.node_id.to_string()),
            None,
            &|node: &mut Node| match node {
                Node::CodeChunk(CodeChunk { execute_status, .. })
                | Node::CodeExpression(CodeExpression { execute_status, .. })
                | Node::Division(Division { execute_status, .. })
                | Node::Span(Span { execute_status, .. }) => match execute_status {
                    Some(ExecuteStatus::Scheduled)
                    | Some(ExecuteStatus::ScheduledPreviouslyFailed) => {
                        *execute_status = self.previous_execute_status.clone()
                    }

                    Some(ExecuteStatus::Running) | Some(ExecuteStatus::RunningPreviouslyFailed) => {
                        *execute_status = Some(ExecuteStatus::Cancelled)
                    }

                    _ => {}
                },
                _ => {}
            },
        )
    }
}

fn get_node_info(node_infos: &BTreeMap<Resource, NodeInfo>, node_id: &str) -> Option<NodeInfo> {
    for node_info in node_infos.values() {
        if node_info.node_id == node_id {
            return Some(node_info.clone());
        }
    }
    None
}

fn handle_cancel_request(
    request: CancelRequest,
    node_infos: &BTreeMap<Resource, NodeInfo>,
    cancellers: &mut HashMap<String, oneshot::Sender<()>>,
    cancelled: &mut Vec<String>,
    patch_request_sender: &UnboundedSender<PatchRequest>,
) -> bool {
    let node_id = request.start;
    let scope = request.scope.unwrap_or(PlanScope::Single);
    tracing::debug!(
        "Handling cancel request for node `{:?}` and scope `{:?}`",
        node_id,
        scope
    );

    match scope {
        PlanScope::Single => {
            let node_id = match node_id {
                Some(node_id) => node_id,
                None => {
                    tracing::error!(
                        "Cancellation scope is `Single` but no node id supplied: ignored"
                    );
                    return false;
                }
            };

            // If the node is currently running cancel it
            if let Some(canceller) = cancellers.remove(&node_id) {
                tracing::debug!("Cancelling running node `{}`", node_id);
                if let Err(..) = canceller.send(()) {
                    tracing::error!(
                        "While attempting to cancel node `{}`: channel closed",
                        node_id
                    );
                } else if let Some(mut node_info) = get_node_info(node_infos, &node_id) {
                    send_patch(
                        patch_request_sender,
                        node_info.set_execute_status_cancelled(),
                        When::Soon,
                    );
                }
            }

            // Add to list of cancelled nodes so if scheduled, does not get run
            cancelled.push(node_id);

            false
        }
        PlanScope::All => {
            let mut node_ids: Vec<String> = node_infos
                .values()
                .map(|node_info| node_info.node_id.clone())
                .collect();

            // Cancel all nodes that are currently running
            tracing::debug!("Cancelling all running nodes");
            let mut patches = Vec::new();
            for node_id in node_ids.iter() {
                if let Some(canceller) = cancellers.remove(node_id) {
                    if let Err(..) = canceller.send(()) {
                        tracing::error!(
                            "While attempting to cancel node `{}`: channel closed",
                            node_id
                        );
                    } else if let Some(mut node_info) = get_node_info(node_infos, node_id) {
                        patches.push(node_info.set_execute_status_cancelled());
                    }
                }
            }
            send_patches(patch_request_sender, patches, When::Soon);

            // Add all nodes in the plan to list of cancelled nodes
            cancelled.append(&mut node_ids);

            true
        }
    }
}
