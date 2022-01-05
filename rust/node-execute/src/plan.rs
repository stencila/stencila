use std::sync::Arc;

use crate::Executable;
use eyre::{bail, Result};
use futures::{stream::FuturesUnordered, StreamExt};
use graph_triples::resources;
use kernels::{KernelSelector, KernelSpace};
use node_address::AddressMap;
use node_patch::{apply, diff, Patch};
use node_pointer::resolve;

use parsers::ParseInfo;
use serde::Serialize;
use stencila_schema::Node;
use tokio::sync::mpsc;

/// A step in an execution plan
///
/// A step is the smallest unit in an execution plan and corresponds to a kernel [`Task`]
/// (but to avoid confusion we use a different name here).
#[derive(Debug, Serialize)]
pub struct Step {
    /// The code node to be executed
    pub(crate) node: resources::Code,

    /// The name of the kernel that the code will be executed in
    ///
    /// If this is `None` it indicates that no kernel capable of executing
    /// the node is available on the machine
    pub(crate) kernel_name: Option<String>,

    /// The parse info for the code
    pub(crate) parse_info: Option<ParseInfo>,

    /// The code will be executed in a fork of the kernel
    ///
    /// Code that has no side effects or who's side effects should be
    /// ignored (i.e. is "@pure") are executed in a fork of the kernel.
    pub(crate) is_fork: bool,
}

/// A stage in an execution plan
///
/// A stage represents a group of [`Step`]s that can be executed concurrently
/// (e.g. because they can be executed in different kernels or a kernel fork)
#[derive(Debug, Default, Serialize)]
pub struct Stage {
    /// The steps to be executed
    pub(crate) steps: Vec<Step>,
}

impl Stage {
    /// Execute the stage
    ///
    /// The [`Step`]s in the stage are executed concurrently. For each step, a [`Patch`] is
    /// generated for the node concerned. When the step is completed, the patch is applied
    /// to the document and published.
    pub async fn execute(
        &self,
        (stage_index, stage_count): (usize, usize),
        node: &mut Node,
        addresses: &AddressMap,
        kernel_space: Arc<KernelSpace>,
        sender: Option<mpsc::Sender<Patch>>,
    ) -> Result<()> {
        // Create a task for each step
        let step_count = self.steps.len();
        let mut tasks = Vec::with_capacity(self.steps.len());
        for (step_index, step) in self.steps.iter().enumerate() {
            // Get the node from the document
            let node_address = addresses.get(&step.node.id).cloned();
            let node_id = Some(step.node.id.clone());
            let pointer = resolve(node, node_address.clone(), node_id.clone())?;

            let pre = pointer.to_node()?;
            let kernel_space = kernel_space.clone();
            let kernel_selector = KernelSelector::new(step.kernel_name.clone(), None, None);
            let parse_info = step.parse_info.clone();
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
                    .execute(
                        &kernel_space,
                        &kernel_selector,
                        parse_info.as_ref(),
                        is_fork,
                    )
                    .await
                {
                    Ok(_) => {
                        let mut patch = diff(&pre, &post);
                        patch.address = node_address;
                        patch.target = node_id;
                        Ok((step_index, patch))
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
            let (step_index, patch) = result??;

            tracing::debug!(
                "Finished step {}/{} of stage {}/{}",
                step_index + 1,
                step_count,
                stage_index + 1,
                stage_count
            );

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

        Ok(())
    }
}

/// The ordering of nodes used when generating a plan
#[derive(Debug, Clone, Serialize)]
pub enum PlanOrdering {
    /// Nodes are executed in the order that they appear in the
    /// document, top to bottom, left to right.
    Appearance,

    /// Nodes are executed in the order that ensures
    /// that the dependencies of a node are executed before it is
    Topological,
}

/// Options for generating a plan
#[derive(Debug, Clone, Serialize)]
pub struct PlanOptions {
    /// The ordering of nodes used when generating the plan
    pub ordering: PlanOrdering,

    /// The maximum step concurrency
    ///
    /// Limits the number of [`Step`]s that can be grouped together in a [`Stage`].
    /// Defaults to the number of logical CPU cores on the current machine.
    pub max_concurrency: usize,
}

impl Default for PlanOptions {
    fn default() -> Self {
        Self {
            ordering: PlanOrdering::Topological,
            max_concurrency: num_cpus::get(),
        }
    }
}

/// An execution plan for a document
#[derive(Debug, Default, Serialize)]
pub struct Plan {
    /// The options used to generate the plan
    pub(crate) options: PlanOptions,

    /// The stages to be executed
    pub(crate) stages: Vec<Stage>,
}

impl Plan {
    /// Execute the plan
    pub async fn execute(
        &self,
        node: &mut Node,
        addresses: &AddressMap,
        kernel_space: Arc<KernelSpace>,
        sender: Option<mpsc::Sender<Patch>>,
    ) -> Result<()> {
        let stage_count = self.stages.len();
        for (stage_index, stage) in self.stages.iter().enumerate() {
            tracing::debug!("Starting stage {}/{}", stage_index + 1, stage_count);
            stage
                .execute(
                    (stage_index, stage_count),
                    node,
                    addresses,
                    kernel_space.clone(),
                    sender.clone(),
                )
                .await?;
            tracing::debug!("Finished stage {}/{}", stage_index + 1, stage_count);
        }
        Ok(())
    }
}
