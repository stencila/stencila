use std::sync::Arc;

use crate::Executable;
use events::publish;
use eyre::{bail, Result};
use futures::{stream::FuturesUnordered, StreamExt};
use graph_triples::resources;
use kernels::{KernelSelector, KernelSpace};
use node_address::Addresses;
use node_patch::{apply, diff};
use node_pointer::resolve;

use serde::Serialize;
use stencila_schema::Node;

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
    pub(crate) kernel: Option<String>,

    /// The code will be executed in a fork of the kernel
    ///
    /// Code that has no side effects or who's side effects should be
    /// ignored (i.e. is "@pure") are executed in a fork of the kernel.
    pub(crate) fork: bool,
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
        node: &mut Node,
        node_id: &str,
        addresses: &Addresses,
        kernel_space: Arc<KernelSpace>,
    ) -> Result<()> {
        // Create a task for each step
        let mut tasks = Vec::with_capacity(self.steps.len());
        for (_step_index, step) in self.steps.iter().enumerate() {
            // Get the node from the document
            let node_address = addresses.get(&step.node.id).cloned();
            let node_id = Some(step.node.id.clone());
            let pointer = resolve(node, node_address.clone(), node_id.clone())?;

            let pre = pointer.to_node()?;
            let kernel_space = kernel_space.clone();
            let kernel_selector = KernelSelector::new(step.kernel.clone(), None, None);

            let task = async move {
                let mut post = pre.clone();
                match post.execute(&kernel_space, &kernel_selector).await {
                    Ok(_) => {
                        let mut patch = diff(&pre, &post);
                        patch.address = node_address;
                        patch.target = node_id;
                        Ok(patch)
                    }
                    Err(error) => bail!(error),
                }
            };
            tasks.push(task);
        }

        // Spawn them all and wait for them to finish
        let mut patches = tasks
            .into_iter()
            .map(tokio::spawn)
            .collect::<FuturesUnordered<_>>();
        while let Some(result) = patches.next().await {
            let patch = result??;

            // If the patch is empty there is nothing else to do
            if patch.is_empty() {
                continue;
            }

            // Apply the patch to the node
            apply(node, &patch)?;

            // Publish the patch
            publish(&["node:", node_id, ":patched"].concat(), patch);
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
        node_id: &str,
        addresses: &Addresses,
        kernel_space: Arc<KernelSpace>,
    ) -> Result<()> {
        // Stages are executed serially
        for (_stage_index, stage) in self.stages.iter().enumerate() {
            stage
                .execute(node, node_id, addresses, kernel_space.clone())
                .await?;
        }

        Ok(())
    }
}
