use crate::{
    plan::{Plan, PlanOptions, PlanOrdering, Stage, Step, StepInfo},
    Executable,
};
use chrono::Utc;
use eyre::{bail, Result};
use futures::{stream::FuturesUnordered, StreamExt};
use graph::Graph;
use graph_triples::{Relations, Resource, ResourceDependencies, ResourceId, ResourceMap};
use kernels::{Kernel, KernelSelector, KernelSpace};
use node_address::AddressMap;
use node_patch::{apply, diff, Patch};
use node_pointer::resolve;
use serde::Serialize;
use std::{
    collections::{BTreeMap, HashSet},
    path::Path,
    sync::Arc,
};
use stencila_schema::Node;
use tokio::sync::mpsc;

/// An execution planner for a document
#[derive(Debug, Clone, Default, Serialize)]
pub struct Planner {
    /// The [`Resource`]s in the document
    ///
    /// This mapping is used for access to more information on a resource than
    /// is available in it's id (which is all that is stored in `appearance_order`
    /// and `topological_order`).
    resources: BTreeMap<ResourceId, Resource>,

    /// The appearance order of [`Resource`]s in the document
    ///
    /// Will include any document node that declares relations (even if there are no relations),
    /// including for example `Link` nodes (but usually only `Code` resources are
    /// of interest here).
    appearance_order: Vec<ResourceId>,

    /// The topological order of [`Resource`]s in, or connected to, the document
    ///
    /// Topological order ensures that the dependencies of a node are executed
    /// before it is. If there are no inter-dependencies between nodes
    /// in a document then the order will be the order that the nodes
    /// appear in the document (i.e. top to bottom, left to right).
    ///
    /// Includes resources other than document nodes (e.g. symbols and files)
    /// so that this order can be used to react to changes in those resources
    /// as well.
    topological_order: Vec<ResourceDependencies>,

    /// The parsing results for code resources
    parse_map: ResourceMap,

    /// The kernels that are available to execute nodes
    kernels: Vec<Kernel>,

    /// Information on when a resource was last executed
    ///
    /// Used to determine which dependencies of a resource need to be executed
    /// before it is.
    executed: BTreeMap<ResourceId, StepInfo>,
}

impl Planner {
    /// Create a new execution planner for a document
    ///    dependency graph)
    #[allow(clippy::ptr_arg)]
    pub async fn new(
        path: &Path,
        relations: &Relations,
        parse_map: ResourceMap,
        kernels: Option<Vec<Kernel>>,
    ) -> Result<Planner> {
        let mut planner = Planner::default();
        planner.update(path, relations, parse_map, kernels).await?;
        Ok(planner)
    }

    /// Update the execution planner for a document
    ///
    /// # Arguments
    ///
    /// - `path`: The path of the document (needed to create a dependency graph)
    ///
    /// - `relations`: The dependency relations between nodes (used to create a
    ///    dependency graph)
    #[allow(clippy::ptr_arg)]
    pub async fn update(
        &mut self,
        path: &Path,
        relations: &Relations,
        parse_map: ResourceMap,
        kernels: Option<Vec<Kernel>>,
    ) -> Result<()> {
        // Get the appearance order from `relations`
        self.appearance_order = relations.iter().map(|(subject, ..)| subject.id()).collect();

        // Create a dependency graph and do a topological sort
        let graph = Graph::from_relations(path, relations);
        self.topological_order = graph.toposort()?;

        // Get the resources from the graph since that already keeps a list of
        // unique resources (including those that are only in relations)
        self.resources = graph.resource_map();

        self.parse_map = parse_map;

        // If no list of kernels was supplied, get it
        self.kernels = match kernels {
            Some(kernels) => kernels,
            None => kernels::available().await?,
        };

        Ok(())
    }

    /// Generate an execution plan
    ///
    /// # Arguments
    ///
    /// - `start`: The node at which the plan should start. If `None` then
    ///            starts at the first node in the document.
    ///
    /// - `options`: Options for the plan
    pub fn plan(&self, start: Option<ResourceId>, options: Option<PlanOptions>) -> Plan {
        let options = options.unwrap_or_default();
        match options.ordering {
            PlanOrdering::Appearance => self.plan_appearance(start, options),
            PlanOrdering::Topological => self.plan_topological(start, options),
        }
    }

    /// Generate an execution plan based on appearance order
    ///
    /// The generated plan ignores the dependencies between resources and
    /// simply executes nodes in the order that they appear in the document.
    ///
    /// # Arguments
    ///
    /// - `start`: The node at which the plan should start. If `None` then
    ///            starts at the first node in the document.
    ///
    /// - `options`: Options for the plan
    pub fn plan_appearance(&self, start: Option<ResourceId>, options: PlanOptions) -> Plan {
        let mut stages: Vec<Stage> = Vec::with_capacity(self.appearance_order.len());
        let mut stage: Stage = Stage::default();
        let mut started = start.is_none();
        for resource_id in &self.appearance_order {
            // Should we start collecting steps?
            if !started {
                started = start.as_ref().map_or(true, |start| start == resource_id)
            }
            if !started {
                continue;
            }

            // Only include `Code` resources (i.e. ignore non-executable `Node`s like `Link` etc)
            let code = match self.resources.get(resource_id) {
                Some(Resource::Code(code)) => code,
                _ => continue,
            };

            // Only include code for which there is a kernel capable of executing it
            let selector = KernelSelector::new(None, code.language.clone(), None);
            let kernel = selector.select(&self.kernels);
            let (kernel_name, kernel_forkable) = match kernel {
                Some(kernel) => (Some(kernel.name.clone()), kernel.forkable),
                None => continue,
            };

            // If (a) the kernel is forkable, (b) the code is `@pure` (inferred or declared),
            // and (c) the maximum concurrency has not been exceeded then execute the step in a fork
            let resource_info = self.parse_map.get(resource_id).cloned();
            let is_pure = resource_info
                .as_ref()
                .map_or(false, |resource_info| resource_info.is_pure());
            let is_fork = kernel_forkable && is_pure && stage.steps.len() < options.max_concurrency;

            // Create the step and add it to the current stage
            let step = Step {
                resource_id: resource_id.clone(),
                code: code.clone(),
                kernel_name,
                resource_info,
                is_fork,
            };
            stage.steps.push(step);

            // If not in a fork, start a new stage.
            if !is_fork {
                stages.push(stage);
                stage = Stage::default();
            }
        }

        // Collect any steps not yet added (e.g. a `CodeExpression` at end of document)
        if !stage.steps.is_empty() {
            stages.push(stage);
        }

        Plan {
            options: PlanOptions {
                ordering: PlanOrdering::Appearance,
                ..options
            },
            stages,
        }
    }

    /// Generate an execution plan based on topological order
    ///
    /// The generated plan executes nodes in the order which ensures that the
    /// dependencies of a node are executed before it is.
    ///
    /// # Arguments
    ///
    /// - `start`: The node at which the plan should start. Only nodes that have `start`
    ///            as a dependency (direct or transitive) will be executed. If `None` then
    ///            the plan applies to all nodes in the document.
    ///
    /// - `options`: Options for the plan
    pub fn plan_topological(&self, start: Option<ResourceId>, options: PlanOptions) -> Plan {
        // First iteration, in topological order, to determine which resources to include
        let mut include = HashSet::new();
        let mut started = start.is_none();
        for resource_dependencies in &self.topological_order {
            let resource_id = &resource_dependencies.id;
            let dependencies = &resource_dependencies.dependencies;

            // Should we start collecting steps?
            if !started {
                started = start.as_ref().map_or(true, |start| start == resource_id);
            }
            if !started {
                continue;
            }

            // Only include resources that are `start` or have `start` in their dependencies
            if let Some(start) = &start {
                if !(start == resource_id || dependencies.contains(start)) {
                    continue;
                }
            }

            // Only include `Code` resources (i.e. ignore `Symbol`s etc which will also be in the dependency
            // graph and therefore in `topological_order` as well)
            match self.resources.get(resource_id) {
                Some(Resource::Code(..)) => include.insert(resource_id),
                _ => continue,
            };

            // Include any dependencies that are not yet included and which have not been
            // executed yet or have a change in hash.
            for dependency in dependencies {
                let execute = match self.executed.get(dependency) {
                    Some(step_info) => {
                        if let Some(resource_info) = self.parse_map.get(dependency) {
                            if let (Some(step_digest), Some(resource_digest)) =
                                (&step_info.execute_digest, &resource_info.execute_digest)
                            {
                                step_digest != resource_digest
                            } else {
                                // No digests available, so execute (perhaps unnecessarily)
                                true
                            }
                        } else {
                            // No parse info available, so execute (perhaps unnecessarily)
                            true
                        }
                    }
                    // Note yet executed (in this session)
                    None => true,
                };
                if execute {
                    include.insert(dependency);
                }
            }
        }

        // Second iteration, in topological order, to create stages and steps
        let mut stages = Vec::with_capacity(include.len());
        let mut stage: Stage = Stage::default();
        for resource_dependencies in &self.topological_order {
            let resource_id = &resource_dependencies.id;

            // Only include resources included above
            if !include.contains(resource_id) {
                continue;
            }

            // Get the `Code` resource to be executed
            let code = match self.resources.get(resource_id) {
                Some(Resource::Code(code)) => code,
                _ => continue,
            };

            // Only execute resources for which there is a kernel capable of executing code
            let selector = KernelSelector::new(None, code.language.clone(), None);
            let kernel = selector.select(&self.kernels);
            let (kernel_name, kernel_forkable) = match kernel {
                Some(kernel) => (Some(kernel.name.clone()), kernel.forkable),
                None => continue,
            };

            // If (a) the kernel is forkable, (b) the code is `@pure` (inferred or declared),
            // and (c) the maximum concurrency has not been exceeded then execute the step in a fork
            let resource_info = self.parse_map.get(resource_id).cloned();
            let is_pure = resource_info
                .as_ref()
                .map_or(false, |resource_info| resource_info.is_pure());
            let is_fork = kernel_forkable && is_pure && stage.steps.len() < options.max_concurrency;

            // Create the step and add it to the current stage
            let step = Step {
                resource_id: resource_id.clone(),
                code: code.clone(),
                kernel_name,
                resource_info,
                is_fork,
            };
            stage.steps.push(step);

            // If not in a fork, start a new stage.
            if !is_fork {
                stages.push(stage);
                stage = Stage::default();
            }
        }

        // Collect any steps not yet added (e.g. a `CodeExpression` at end of document)
        if !stage.steps.is_empty() {
            stages.push(stage);
        }

        Plan {
            options: PlanOptions {
                ordering: PlanOrdering::Topological,
                ..options
            },
            stages,
        }
    }

    /// Execute a plan
    pub async fn execute(
        &mut self,
        node: &mut Node,
        addresses: &AddressMap,
        kernel_space: Arc<KernelSpace>,
        sender: Option<mpsc::Sender<Patch>>,
        start: Option<ResourceId>,
        plan_options: Option<PlanOptions>,
    ) -> Result<()> {
        let plan = self.plan(start, plan_options);

        let stage_count = plan.stages.len();
        for (stage_index, stage) in plan.stages.iter().enumerate() {
            tracing::debug!("Starting stage {}/{}", stage_index + 1, stage_count);

            // Create a task for each step
            let step_count = stage.steps.len();
            let mut tasks = Vec::with_capacity(step_count);
            for (step_index, step) in stage.steps.iter().enumerate() {
                // Get the node from the document
                let node_address = addresses.get(&step.code.id).cloned();
                let node_id = Some(step.code.id.clone());
                let pointer = resolve(node, node_address.clone(), node_id.clone())?;

                let pre = pointer.to_node()?;
                let kernel_space = kernel_space.clone();
                let kernel_selector = KernelSelector::new(step.kernel_name.clone(), None, None);
                let resource_id = step.resource_id.clone();
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
                        .execute(
                            &kernel_space,
                            &kernel_selector,
                            resource_info.as_ref(),
                            is_fork,
                        )
                        .await
                    {
                        Ok(_) => {
                            let mut patch = diff(&pre, &post);
                            patch.address = node_address;
                            patch.target = node_id;

                            let step_info = StepInfo {
                                finished: Utc::now(),
                                execute_digest: resource_info
                                    .map_or(Some("".to_string()), |resource_info| {
                                        resource_info.execute_digest
                                    }),
                            };

                            Ok((step_index, resource_id, step_info, patch))
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
                let (step_index, resource_id, step_info, patch) = result??;

                tracing::debug!(
                    "Finished step {}/{} of stage {}/{} for {}",
                    step_index + 1,
                    step_count,
                    stage_index + 1,
                    stage_count,
                    resource_id
                );

                // Update the record of executed resources
                self.executed.insert(resource_id, step_info);

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
}
