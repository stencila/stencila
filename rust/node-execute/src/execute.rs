use std::{path::Path, sync::Arc};

use eyre::{bail, eyre, Result};
use futures::stream::{FuturesUnordered, StreamExt};
use graph::{Graph, Plan, PlanOptions};
use graph_triples::Resource;
use kernels::{Kernel, KernelSelector, KernelSpace};
use node_address::AddressMap;
use node_patch::{apply, diff, Patch};
use node_pointer::resolve;
use stencila_schema::Node;
use tokio::sync::mpsc;

use crate::{compile, Executable};

/// Execute a node
///
/// Executing a node involves:
///
/// - [`compile`]ing it to get a set of [`Addresses`] and [`Relations`]
///
/// - generating an execution plan, based on that set of relations
///
/// - executing the plan
///
#[tracing::instrument(skip(node, kernel_space))]
pub async fn execute(
    node: &mut Node,
    path: &Path,
    project: &Path,
    start: Option<Resource>,
    kernels: Option<Vec<Kernel>>,
    kernel_space: Option<Arc<KernelSpace>>,
    options: Option<PlanOptions>,
) -> Result<()> {
    let (addresses, resources) = compile(node, path, project)?;
    let graph = Graph::from_resource_infos(path, resources)?;
    let plan = graph.plan(start, kernels, options).await?;
    let kernel_space = kernel_space.unwrap_or_default();
    execute_plan(&plan, node, &addresses, kernel_space, None).await
}

/// Execute a plan
pub async fn execute_plan(
    plan: &Plan,
    node: &mut Node,
    addresses: &AddressMap,
    kernel_space: Arc<KernelSpace>,
    sender: Option<mpsc::Sender<Patch>>,
) -> Result<()> {
    let stage_count = plan.stages.len();
    for (stage_index, stage) in plan.stages.iter().enumerate() {
        tracing::debug!("Starting stage {}/{}", stage_index + 1, stage_count);

        // Create a task for each step
        let step_count = stage.steps.len();
        let mut tasks = Vec::with_capacity(step_count);
        for (step_index, step) in stage.steps.iter().enumerate() {
            // Get the node from the document
            let node_id = step
                .resource_info
                .resource
                .node_id()
                .ok_or_else(|| eyre!("Expected to get code id for resource"))?;
            let node_address = addresses.get(&node_id).cloned();
            let pointer = resolve(node, node_address.clone(), Some(node_id.clone()))?;

            let pre = pointer.to_node()?;
            let kernel_space = kernel_space.clone();
            let kernel_selector = KernelSelector::new(step.kernel_name.clone(), None, None);
            let resource = step.resource_info.resource.clone();
            let resource_info = step.resource_info.clone();
            let is_fork = step.is_fork;

            let task = async move {
                tracing::debug!(
                    "Starting step {}/{} of stage {}/{}",
                    step_index + 1,
                    step_count,
                    stage_index + 1,
                    stage_count
                );

                let mut post = pre.clone();
                match post
                    .execute(&kernel_space, &kernel_selector, &resource_info, is_fork)
                    .await
                {
                    Ok(_) => {
                        let mut patch = diff(&pre, &post);
                        patch.address = node_address;
                        patch.target = Some(node_id);
                        Ok((step_index, resource, patch))
                    }
                    Err(error) => bail!(error),
                }
            };
            tasks.push(task);
        }

        // Spawn them all and wait for them to finish
        let mut results = tasks
            .into_iter()
            .map(tokio::spawn)
            .collect::<FuturesUnordered<_>>();
        while let Some(result) = results.next().await {
            let (step_index, _resource, patch) = result??;

            tracing::debug!(
                "Finished step {}/{} of stage {}/{}",
                step_index + 1,
                step_count,
                stage_index + 1,
                stage_count
            );

            // Update the record of executed resources
            // TODO update graph
            // self.executed.insert(resource_id, step_info);

            // If the patch is empty there is nothing else to do
            if patch.is_empty() {
                continue;
            }

            // Apply the patch to the node
            apply(node, &patch)?;

            // Send the patch if a sender was supplied
            if let Some(sender) = &sender {
                if let Err(error) = sender.send(patch).await {
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
