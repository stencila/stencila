use crate::{
    plan::{PlanOrdering, Stage, Step},
    Plan, PlanOptions,
};
use eyre::Result;
use graph::Graph;
use graph_triples::{Relations, Resource, ResourceDependencies, ResourceId};
use kernels::{Kernel, KernelSelector};
use serde::Serialize;
use std::{collections::BTreeMap, path::Path};

/// An execution planner for a document
#[derive(Debug, Default, Serialize)]
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

    /// The kernels that are available to execute nodes
    kernels: Vec<Kernel>,
}

impl Planner {
    /// Create an execution planner for a document
    ///
    /// # Arguments
    ///
    /// - `path`: The path of the document (needed to create a dependency graph)
    ///
    /// - `relations`: The dependency relations between nodes (used to create a
    ///    dependency graph)
    #[allow(clippy::ptr_arg)]
    pub fn new(path: &Path, relations: &Relations, kernels: &[Kernel]) -> Result<Planner> {
        // Store the appearance order from `relations`
        let appearance_order = relations.iter().map(|(subject, ..)| subject.id()).collect();

        // Create a dependency graph and do a topological sort
        let graph = Graph::from_relations(path, relations);
        let topological_order = graph.toposort()?;

        // Get the resources from the graph since that already keeps a list of
        // unique resources (including those that are only in relations)
        let resources = graph.resource_map();

        Ok(Planner {
            resources,
            appearance_order,
            topological_order,
            kernels: kernels.into(),
        })
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
            let fork = kernel_forkable && code.pure && stage.steps.len() < options.max_concurrency;

            // Create the step and add it to the current stage
            let step = Step {
                node: code.clone(),
                kernel: kernel_name,
                fork,
            };
            stage.steps.push(step);

            // If not in a fork, start a new stage.
            if !fork {
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
        let mut stages = Vec::with_capacity(self.topological_order.len());
        let mut stage: Stage = Stage::default();
        let mut started = start.is_none();
        for resource_dependencies in &self.topological_order {
            let resource_id = &resource_dependencies.id;

            // Should we start collecting steps?
            if !started {
                started = start.as_ref().map_or(true, |start| start == resource_id)
            }
            if !started {
                continue;
            }

            // Only include resources that have `start` in their dependencies
            if let Some(start) = &start {
                if !resource_dependencies.dependencies.contains(start) {
                    continue;
                }
            }

            // Only include `Code` resources (i.e. ignore `Symbol`s etc which will also be in the dependency
            // graph and therefore in `topological_order` as well)
            let code = match self.resources.get(resource_id) {
                Some(Resource::Code(code)) => code,
                _ => continue,
            };

            // Find a kernel capable of executing code
            let selector = KernelSelector::new(None, code.language.clone(), None);
            let kernel = selector.select(&self.kernels);
            let (kernel_name, kernel_forkable) = match kernel {
                Some(kernel) => (Some(kernel.name.clone()), kernel.forkable),
                None => continue,
            };

            // If (a) the kernel is forkable, (b) the code is `@pure` (inferred or declared),
            // and (c) the maximum concurrency has not been exceeded then execute the step in a fork
            let fork = kernel_forkable && code.pure && stage.steps.len() < options.max_concurrency;

            // Create the step and add it to the current stage
            let step = Step {
                node: code.clone(),
                kernel: kernel_name,
                fork,
            };
            stage.steps.push(step);

            // If not in a fork, start a new stage.
            if !fork {
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
}
