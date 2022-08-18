use std::{
    cmp::Ordering,
    collections::{BTreeMap, HashMap, HashSet},
    hash::Hasher,
    path::{Path, PathBuf},
};

use petgraph::{
    graph::NodeIndex,
    stable_graph::StableGraph,
    visit::{self, EdgeRef, IntoEdgeReferences, IntoNodeReferences},
    EdgeDirection::Incoming,
};
use schemars::{
    gen::SchemaGenerator,
    schema::{Schema, SchemaObject},
    schema_for, JsonSchema,
};

use common::{
    eyre::{bail, eyre, Result},
    serde::{self, ser::SerializeMap, Serialize},
    serde_json::{self, json},
    serde_with::skip_serializing_none,
    serde_yaml,
    strum::Display,
};
use graph_triples::{
    direction, relations, stencila_schema::CodeChunkExecuteAuto, Direction, Pairs, Relation,
    Resource, ResourceInfo, Triple,
};
use hash_utils::seahash;
use kernels::{Kernel, KernelSelector};
use path_utils::{path_slash::PathExt, pathdiff};
use utils::some_string;

use crate::{Plan, PlanOptions, PlanOrdering, PlanStage, PlanTask};

/// A dependency graph for a project or document
#[derive(Debug, Default, Clone)]
pub struct Graph {
    /// The path of the project or document that this graph is for
    ///
    /// Primarily used to make file paths relative in visualizations and
    /// if ever persisting the graph.
    path: PathBuf,

    /// Information on each of the resources in the graph
    ///
    /// Uses a `BTreeMap` for determinism.
    resources: BTreeMap<Resource, ResourceInfo>,

    /// The appearance order of [`Resource`]s in a document
    ///
    /// Only includes resources added, not those in relations.
    appearance_order: Vec<Resource>,

    /// The topological order of [`Resource`]s in a document
    ///
    /// Only includes all resources in the graph.
    topological_order: Vec<Resource>,

    /// The indices of the resources in the graph
    ///
    /// It is necessary to store [`NodeIndex`] for each resource
    /// so we can keep track of which resources are already in the
    /// graph and re-use their index if they are.
    indices: HashMap<Resource, NodeIndex>,

    /// The graph itself
    ///
    /// Use a `petgraph::StableGraph` so that nodes can be added and removed
    /// without changing node indices.
    graph: StableGraph<Resource, Relation>,
}

impl Serialize for Graph {
    /// Custom serialization to strip prefix from paths, add stable node indices,
    /// and exclude properties that are included by default by `petgraph` (e.g `node_holes`).
    ///
    /// Our general approach is to keep paths absolute whilst in memory and only convert to
    /// relative when necessary (e.g. visualizations). See also `Graph::to_dot`.
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let nodes: Vec<serde_json::Value> = self
            .graph
            .node_references()
            .map(|(index, resource)| {
                let mut obj = serde_json::to_value(resource).expect("To be able to serialize");
                let obj = obj.as_object_mut().expect("To be an object");

                // Strip prefix from paths
                if let Some(path) = match resource {
                    Resource::Symbol(symbol) => Some(symbol.path.clone()),
                    Resource::Node(node) => Some(node.path.clone()),
                    Resource::File(file) => Some(file.path.clone()),
                    _ => None,
                } {
                    let path = path
                        .strip_prefix(&self.path)
                        .unwrap_or(&path)
                        .to_slash_lossy();
                    obj.insert("path".to_string(), json!(path));
                }

                // Merge in fields from resource info but skip info already there or implicit
                // in the graph
                if let Some(resource_info) = self.resources.get(resource) {
                    if let Some(dependencies) = &resource_info.dependencies {
                        obj.insert("dependencies".to_string(), json!(dependencies.len()));
                    }
                    if let Some(dependents) = &resource_info.dependents {
                        obj.insert("dependents".to_string(), json!(dependents.len()));
                    }
                    if let Some(depth) = resource_info.depth {
                        obj.insert("depth".to_string(), json!(depth));
                    }
                    if let Some(execute_auto) = &resource_info.execute_auto {
                        obj.insert("execute_auto".to_string(), json!(execute_auto));
                    }
                    if let Some(execute_pure) = resource_info.execute_pure {
                        obj.insert("execute_pure".to_string(), json!(execute_pure));
                    }
                    if let Some(compile_digest) = &resource_info.compile_digest {
                        obj.insert("compile_digest".to_string(), json!(compile_digest));
                    }
                    if let Some(execute_digest) = &resource_info.execute_digest {
                        obj.insert("execute_digest".to_string(), json!(execute_digest));
                    }
                }

                obj.insert("index".to_string(), json!(index));

                json!(obj)
            })
            .collect();

        let edges: Vec<serde_json::Value> = self
            .graph
            .edge_references()
            .map(|edge| -> serde_json::Value {
                json!({
                    "from": edge.source(),
                    "to": edge.target(),
                    "relation": edge.weight()
                })
            })
            .collect();

        let mut map = serializer.serialize_map(Some(2))?;
        map.serialize_entry("nodes", &nodes)?;
        map.serialize_entry("edges", &edges)?;
        map.end()
    }
}

/// The available graph serialization formats
pub const FORMATS: [&str; 3] = ["dot", "json", "yaml"];

impl Graph {
    /// Create a new, empty graph
    pub fn new<P: AsRef<Path>>(path: P) -> Graph {
        Graph {
            path: PathBuf::from(path.as_ref()),
            resources: BTreeMap::new(),
            appearance_order: Vec::new(),
            topological_order: Vec::new(),
            indices: HashMap::new(),
            graph: StableGraph::new(),
        }
    }

    /// Make a path relative to the path of the graph
    pub fn relative_path(&self, path: &Path, is_doc: bool) -> PathBuf {
        let base = match is_doc {
            true => self.path.parent().unwrap_or(&self.path),
            false => &self.path,
        };
        pathdiff::diff_paths(path, &base).unwrap_or_else(|| path.to_path_buf())
    }

    /// Create a graph from a set of [`ResourceInfo`] objects
    pub fn from_resource_infos<P: AsRef<Path>>(
        path: P,
        resource_infos: Vec<ResourceInfo>,
    ) -> Result<Self> {
        let mut graph = Graph::new(path);
        graph.add_resource_infos(resource_infos)?;
        Ok(graph)
    }

    /// Create a graph from set of dependency relations
    pub fn from_relations<P: AsRef<Path>>(path: P, relations: &[(Resource, Pairs)]) -> Self {
        let mut graph = Graph::new(path);
        graph.add_relations(relations);
        graph
    }

    /// Add a resource to the graph
    ///
    /// Add a resource, and optionally information on the resource, to the graph. If `resource_info` is
    /// `None` then will calculate it for that resource type.
    pub fn add_resource(
        &mut self,
        resource: Resource,
        resource_info: Option<ResourceInfo>,
    ) -> NodeIndex {
        if let Some(index) = self.indices.get(&resource) {
            *index
        } else {
            let resource_info = resource_info.unwrap_or_else(|| resource.resource_info());
            self.resources.insert(resource.clone(), resource_info);

            let index = self.graph.add_node(resource.clone());
            self.indices.insert(resource, index);
            index
        }
    }

    /// Add a triple to the graph
    pub fn add_triple(&mut self, (subject, relation, object): Triple) {
        let subject = if let Some(index) = self.indices.get(&subject) {
            *index
        } else {
            let index = self.graph.add_node(subject.clone());
            self.indices.insert(subject, index);
            index
        };

        let object = if let Some(index) = self.indices.get(&object) {
            *index
        } else {
            let index = self.graph.add_node(object.clone());
            self.indices.insert(object, index);
            index
        };

        let (from, to) = match direction(&relation) {
            Direction::From => (object, subject),
            Direction::To => (subject, object),
        };

        self.graph.add_edge(from, to, relation);
    }

    /// Add a set of triples to the graph
    pub fn add_triples(&mut self, triples: Vec<Triple>) {
        triples
            .into_iter()
            .for_each(|triple| self.add_triple(triple))
    }

    /// Add a set of relations to the graph
    ///
    /// Each subject resource in the set will be added to the graph even if it has
    /// no relations with other objects.
    pub fn add_relations(&mut self, relations: &[(Resource, Pairs)]) {
        for (subject, pairs) in relations {
            if pairs.is_empty() {
                self.add_resource(subject.clone(), None);
                continue;
            }

            for (relation, object) in pairs {
                self.add_triple((subject.clone(), relation.clone(), object.clone()));
            }
        }
    }

    /// Get [`ResourceInfo`] objects in the graph
    pub fn get_resource_infos(&self) -> &BTreeMap<Resource, ResourceInfo> {
        &self.resources
    }

    /// Get a [`ResourceInfo`] object for a [`Resource`] in the graph
    pub fn get_resource_info(&self, resource: &Resource) -> Result<&ResourceInfo> {
        self.resources
            .get(resource)
            .ok_or_else(|| eyre!("Graph has no info for resource: {}", resource.resource_id(),))
    }

    /// Find a [`ResourceInfo`] object for a [`Resource`] that is equal to one in the graph
    ///
    /// Even though `self.resources` is keyed by `Resource`, it seems to be necessary to
    /// use `find` (and the equality operator) when attempting to get resources that are not
    /// aleady a key in `resources`.
    pub fn find_resource_info(&self, resource: &Resource) -> Result<&ResourceInfo> {
        self.resources
            .values()
            .find(|resource_info| resource_info.resource == *resource)
            .ok_or_else(|| {
                eyre!(
                    "Could not find info for resource: {}",
                    resource.resource_id(),
                )
            })
    }

    /// Add a set of [`ResourceInfo`] objects to the graph
    pub fn add_resource_infos(&mut self, resource_infos: Vec<ResourceInfo>) -> Result<()> {
        for resource_info in resource_infos.into_iter() {
            let subject = resource_info.resource.clone();
            let relations = resource_info.relations.clone();

            // Add the subject to the appearance order
            self.appearance_order.push(subject.clone());

            // Add the subject resource (if it is not already)
            let subject_index = self.add_resource(subject, Some(resource_info));

            // Add all the `Resource`s and `Relation`s that are in `relations`
            // for the resource
            if let Some(relations) = relations {
                for (relation, object) in &relations {
                    // Skip adding `Use` relations where the resource was declared or assigned
                    // in the same resource. These are unnecessary and create a cyclic, one-to-one dependency.
                    let mut skip = false;
                    if matches!(relation, Relation::Uses(..)) {
                        for (other_relation, other_object) in &relations {
                            if matches!(
                                other_relation,
                                Relation::Declares(..) | Relation::Assigns(..)
                            ) && other_object == object
                            {
                                skip = true;
                                break;
                            }
                        }
                    }
                    if skip {
                        continue;
                    }

                    // Add the object resource (if it is not already)
                    let object_index = self.add_resource(object.clone(), None);

                    // Add an edge
                    let (from, to) = match direction(relation) {
                        Direction::From => (object_index, subject_index),
                        Direction::To => (subject_index, object_index),
                    };
                    self.graph.add_edge(from, to, relation.clone());
                }
            }
        }
        self.update(None)?;
        Ok(())
    }

    /// Update the graph
    ///
    /// # Arguments
    ///
    /// - `start`: The graph resource from which the update should be started
    ///   (in topological order); if `None` will update all resources in the graph.
    pub fn update(&mut self, start: Option<Resource>) -> Result<()> {
        let graph = &self.graph;

        // Always update the topological order given that we're doing a topological sort anyway
        self.topological_order.clear();

        let mut started = start.is_none();
        let mut topo = visit::Topo::new(&graph);
        while let Some(node_index) = topo.next(&graph) {
            let resource = &graph[node_index];
            self.topological_order.push(resource.clone());

            // Should we start updating resources?
            if !started {
                if let Some(start) = &start {
                    started = start == resource;
                }
            }
            if !started {
                continue;
            }

            // Calculate stuff from dependencies
            let incomings = graph.neighbors_directed(node_index, Incoming);
            let mut dependencies = Vec::new();
            let mut depth = 0;
            let mut dependencies_digest = seahash::SeaHasher::new();
            let mut dependencies_stale = 0;
            let mut dependencies_failed = 0;
            for incoming_index in incomings {
                let dependency = &graph[incoming_index];
                let dependency_info = self.get_resource_info(dependency)?;

                // Update list of dependencies
                if let Some(dependency_dependencies) = &dependency_info.dependencies {
                    for other in dependency_dependencies {
                        if !dependencies.contains(other) {
                            dependencies.push(other.clone())
                        }
                    }
                }
                if !dependencies.contains(dependency) {
                    dependencies.push(dependency.clone());
                }

                // Update depth
                let dependency_depth = dependency_info.depth.unwrap_or_default();
                if dependency_depth + 1 > depth {
                    depth = dependency_depth + 1
                }

                let compile_digest = dependency_info
                    .compile_digest
                    .as_ref()
                    .ok_or_else(|| eyre!("Dependency has no compile_digest"))?;

                // Update the digest of dependencies using semantic digest
                // (or content digest if that is empty) and dependencies digest.
                dependencies_digest.write_u64(match compile_digest.semantic_digest != 0 {
                    true => compile_digest.semantic_digest,
                    false => compile_digest.content_digest,
                });
                dependencies_digest.write_u64(compile_digest.dependencies_digest);

                // Update the number of `Code` dependencies that are stale and failed. This is transitive,
                // so if the resource is not code (e.g. a `Symbol`) then add its value.
                if matches!(dependency, Resource::Code(..)) {
                    if dependency_info.is_stale() {
                        dependencies_stale += 1;
                    }
                    if dependency_info.is_fail() {
                        dependencies_failed += 1;
                    }
                } else {
                    dependencies_stale += compile_digest.dependencies_stale;
                    dependencies_failed += compile_digest.dependencies_failed;
                }
            }

            // If there are no dependencies then `dependencies_digest` is an empty string
            let dependencies_digest = match !dependencies.is_empty() {
                true => dependencies_digest.finish(),
                false => 0,
            };

            // Get the resource info we're about to update
            let resource_info = self
                .resources
                .get_mut(resource)
                .ok_or_else(|| eyre!("No info for resource"))?;

            resource_info.dependencies = Some(dependencies);
            resource_info.depth = Some(depth);

            // Update the compile digest, or create one if there isn't one already.
            match resource_info.compile_digest.as_mut() {
                Some(compile_digest) => {
                    compile_digest.dependencies_digest = dependencies_digest;
                    compile_digest.dependencies_stale = dependencies_stale;
                    compile_digest.dependencies_failed = dependencies_failed;
                }
                None => {
                    let mut compile_digest = resource.digest();
                    compile_digest.dependencies_digest = dependencies_digest;
                    compile_digest.dependencies_stale = dependencies_stale;
                    compile_digest.dependencies_failed = dependencies_failed;
                    resource_info.compile_digest = Some(compile_digest);
                }
            }
        }

        // Populate dependents and sort dependencies and dependents.
        let mut dependents_map: HashMap<Resource, HashSet<Resource>> = HashMap::new();
        for (resource, resource_info) in self.resources.iter_mut() {
            if let Some(dependencies) = &mut resource_info.dependencies {
                // Map dependencies into dependents
                for dependency in dependencies.iter() {
                    dependents_map
                        .entry(dependency.clone())
                        .or_insert_with(HashSet::new)
                        .insert(resource.clone());
                }

                dependencies.sort_by(|a, b| match (a, b) {
                    // Sort code dependencies by appearance order. The order is not important for
                    // dependency analysis or plan generation but appearance order is better for
                    // user interfaces.
                    (Resource::Code(..), Resource::Code(..)) => {
                        let a = self
                            .appearance_order
                            .iter()
                            .position(|resource| resource == a);
                        let b = self
                            .appearance_order
                            .iter()
                            .position(|resource| resource == b);
                        a.cmp(&b)
                    }
                    // Other types of dependencies (e.g. `File`, `Symbol`) come last.
                    (Resource::Code(..), ..) => Ordering::Less,
                    (.., Resource::Code(..)) => Ordering::Greater,
                    _ => a.cmp(b),
                });
            }
        }
        for (resource, dependents) in dependents_map {
            if let Some(resource_info) = self.resources.get_mut(&resource) {
                let mut dependents = Vec::from_iter(dependents);
                dependents.sort_by(|a, b| match (a, b) {
                    // As for dependencies, sort code dependents by appearance order.
                    (Resource::Code(..), Resource::Code(..)) => {
                        let a = self
                            .appearance_order
                            .iter()
                            .position(|resource| resource == a);
                        let b = self
                            .appearance_order
                            .iter()
                            .position(|resource| resource == b);
                        a.cmp(&b)
                    }
                    // Other types of dependencies (e.g. `File`, `Symbol`) come last.
                    (Resource::Code(..), ..) => Ordering::Greater,
                    (.., Resource::Code(..)) => Ordering::Less,
                    _ => a.cmp(b),
                });
                resource_info.dependents = Some(dependents)
            }
        }

        Ok(())
    }

    /// Determine whether a task should be run in a kernel fork or not
    ///
    /// Execute the task in a fork if,
    /// - the kernel is forkable, and
    /// - the node is a `CodeChunk`
    /// - the node's code is `@pure` (inferred or declared), and
    /// - the maximum concurrency has not been exceeded, and
    fn should_run_in_fork(
        kernel_forkable: bool,
        resource_info: &ResourceInfo,
        stage_tasks: usize,
        options: &PlanOptions,
    ) -> bool {
        kernel_forkable
            && resource_info.resource.node_type() == Some("CodeChunk")
            && resource_info.is_pure()
            && stage_tasks < options.max_concurrency.saturating_sub(1)
    }

    /// Determine whether a task needs to be run in a new stage
    ///
    /// Execute the task in a new stage if any of the existing tasks
    /// are not in a fork
    fn needs_new_stage(tasks: &[PlanTask]) -> bool {
        tasks.iter().any(|task| !task.is_fork)
    }

    /// Generate an execution plan for the graph
    ///
    /// # Arguments
    ///
    /// - `start`: The node at which the plan should start. If `None` then
    ///            starts at the first node in the document.
    ///
    /// - `options`: Options for the plan
    pub async fn plan(
        &self,
        start: Option<Resource>,
        kernels: Option<Vec<Kernel>>,
        options: Option<PlanOptions>,
    ) -> Result<Plan> {
        let kernels = match kernels {
            Some(kernels) => kernels,
            None => kernels::available().await,
        };

        let options = options.unwrap_or_default();
        match options.ordering {
            PlanOrdering::Single => self.plan_single(start, kernels, options),
            PlanOrdering::Appearance => self.plan_appearance(start, kernels, options),
            PlanOrdering::Topological => self.plan_topological(start, kernels, options),
        }
    }

    /// Generate an execution plan for a single node
    ///
    /// The start resource must be supplied and be a code node with a kernel
    /// capable of executing it. If the kernel is forkable, and the code is
    /// `@pure` (inferred or declared), then the code will be executed in a fork
    /// to avoid any unintended side-effects.
    ///
    /// # Arguments
    ///
    /// - `start`: The node at which the plan should start. If `None` then
    ///            starts at the first node in the document.
    ///
    /// - `kernels`: The kernels available to execute the plan
    ///
    /// - `options`: Options for the plan
    pub fn plan_single(
        &self,
        start: Option<Resource>,
        kernels: Vec<Kernel>,
        options: PlanOptions,
    ) -> Result<Plan> {
        let start = match start {
            Some(start) => start,
            None => {
                bail!("A resource must be supplied for plan ordering `Single`")
            }
        };

        // Get the `ResourceInfo` and `Resource` for `start` which have more information
        // (such as the code language etc)
        let resource_info = self.find_resource_info(&start)?.clone();
        let resource = &resource_info.resource;

        let code = match resource {
            Resource::Code(code) => code,
            _ => bail!("The resource must be a `Code` node for plan ordering `Simple`"),
        };

        let kernel = KernelSelector::new(None, code.language.clone(), None).select(&kernels);
        let (kernel_name, kernel_forkable) = match kernel {
            Some(kernel) => (Some(kernel.name.clone()), kernel.forkable),
            None => bail!("There is no kernel available capable of executing the code"),
        };

        let is_fork = Self::should_run_in_fork(kernel_forkable, &resource_info, 0, &options);

        Ok(Plan {
            options: PlanOptions {
                ordering: PlanOrdering::Single,
                ..options
            },
            stages: vec![PlanStage {
                tasks: vec![PlanTask {
                    resource_info,
                    kernel_name,
                    is_fork,
                }],
            }],
        })
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
    /// - `kernels`: The kernels available to execute the plan
    ///
    /// - `options`: Options for the plan
    pub fn plan_appearance(
        &self,
        start: Option<Resource>,
        kernels: Vec<Kernel>,
        options: PlanOptions,
    ) -> Result<Plan> {
        let mut stages: Vec<PlanStage> = Vec::with_capacity(self.appearance_order.len());
        let mut stage: PlanStage = PlanStage::default();
        let mut started = start.is_none();
        for resource in &self.appearance_order {
            // Should we start collecting tasks?
            if !started {
                if let Some(start) = &start {
                    started = start == resource;
                }
            }
            if !started {
                continue;
            }

            // Only include `Code` resources (i.e. ignore non-executable resources like `Link` nodes etc)
            let code = match resource {
                Resource::Code(code) => code,
                _ => continue,
            };

            // Only include code for which there is a kernel capable of executing it
            let selector = KernelSelector::new(None, code.language.clone(), None);
            let kernel = selector.select(&kernels);
            let (kernel_name, kernel_forkable) = match kernel {
                Some(kernel) => (Some(kernel.name.clone()), kernel.forkable),
                None => continue,
            };

            let resource_info = self.get_resource_info(resource)?;

            // If this is not the explicitly executed resource `start`
            // and `autorun == Never` then exclude it and any following resources
            if start.is_some()
                && Some(resource) != start.as_ref()
                && matches!(
                    resource_info.execute_auto,
                    Some(CodeChunkExecuteAuto::Never)
                )
            {
                break;
            }

            // Determine if the taks should run in a fork
            let is_fork = Self::should_run_in_fork(
                kernel_forkable,
                resource_info,
                stage.tasks.len(),
                &options,
            );

            // Create the task
            let task = PlanTask {
                resource_info: resource_info.clone(),
                kernel_name,
                is_fork,
            };

            // Add the task to a new, or the current, stage
            if Self::needs_new_stage(&stage.tasks) {
                if !stage.tasks.is_empty() {
                    stages.push(stage);
                    stage = PlanStage::default();
                }
                stages.push(PlanStage { tasks: vec![task] });
            } else {
                stage.tasks.push(task);
            }
        }

        // Collect any tasks not yet added (e.g. a `CodeExpression` at end of document)
        if !stage.tasks.is_empty() {
            stages.push(stage);
        }

        Ok(Plan {
            options: PlanOptions {
                ordering: PlanOrdering::Appearance,
                ..options
            },
            stages,
        })
    }

    /// Generate an execution plan based on topological order
    ///
    /// The generated plan executes nodes in the order which ensures that the
    /// dependencies of a node are executed before it is.
    ///
    /// # Arguments
    ///
    /// - `start`: The node at which the plan should start. Nodes that are upstream dependencies
    ///            of `start` and are stale (and not `executeAuto == Never`) or are downstream
    ///            dependent of `start` (and not `executeAuto == Never`) will be executed.
    ///            If `None` then the plan includes all nodes in the document.
    ///
    /// - `kernels`: The kernels available to execute the plan
    ///
    /// - `options`: Options for the plan
    pub fn plan_topological(
        &self,
        start: Option<Resource>,
        kernels: Vec<Kernel>,
        options: PlanOptions,
    ) -> Result<Plan> {
        // First iteration, in topological order, to determine which resources to include
        let mut included = HashSet::new();
        let mut excluded = HashSet::new();
        let mut started = start.is_none();
        for resource in &self.topological_order {
            // Should we start collecting tasks?
            if !started {
                if let Some(start) = &start {
                    started = start == resource;
                }
            }
            if !started {
                continue;
            }

            // Skip non-`Code` resources (i.e ignore `Symbol`s etc which will also be in the dependency
            // graph and therefore in `topological_order` as well)
            if !matches!(resource, Resource::Code(..)) {
                continue;
            }

            let start = if let Some(start) = &start {
                start
            } else {
                // If `start` is None (i.e. whole document run) always include the resource and continue
                included.insert(resource);
                continue;
            };

            // Other resources will be included if:
            //  - they are a code resource
            //  - does not have `autorun == Never`
            //  - does not have any dependencies that are stale and have `autorun == Never`
            //    (there is no point running these) unless the dependency is `start`
            let mut should_include = |resource_info: &ResourceInfo| -> Result<bool> {
                // Cache set of excluded resources in particular to avoid the following loop
                if excluded.contains(&resource_info.resource) {
                    return Ok(false);
                }

                if !matches!(resource_info.resource, Resource::Code(..))
                    || matches!(
                        resource_info.execute_auto,
                        Some(CodeChunkExecuteAuto::Never)
                    )
                {
                    excluded.insert(resource_info.resource.clone());
                    return Ok(false);
                }

                for dependency in resource_info.dependencies.iter().flatten() {
                    if dependency != start && matches!(dependency, Resource::Code(..)) {
                        let dependency_info = self.get_resource_info(dependency)?;
                        if dependency_info.is_stale()
                            && matches!(
                                dependency_info.execute_auto,
                                Some(CodeChunkExecuteAuto::Never)
                            )
                        {
                            excluded.insert(resource_info.resource.clone());
                            return Ok(false);
                        }
                    }
                }

                Ok(true)
            };

            let resource_info = self.get_resource_info(resource)?;
            let dependencies: Vec<_> = resource_info.dependencies.iter().flatten().collect();

            if resource == start {
                // Resource is start so always include
                included.insert(resource);
            } else if dependencies.contains(&start) {
                // Resource has start as a dependency so maybe include (if not blocked by other dependencies)
                if should_include(resource_info)? {
                    included.insert(resource);
                } else {
                    continue;
                }
            } else {
                // Resource is not `start` and does not depend upon it
                continue;
            };

            // If the resource was included then ensure any of its dependency that are stale or are
            // `autorun == Always` are also included, as well as dependents of those dependencies
            for dependency in dependencies {
                let dependency_info = self.get_resource_info(dependency)?;
                if (matches!(
                    dependency_info.execute_auto,
                    Some(CodeChunkExecuteAuto::Always)
                ) || dependency_info.is_stale())
                    && should_include(dependency_info)?
                {
                    included.insert(dependency);

                    for dependent in dependency_info.dependents.iter().flatten() {
                        let dependent_info = self.get_resource_info(dependent)?;
                        if should_include(dependent_info)? {
                            included.insert(dependent);
                        }
                    }
                }
            }
        }

        // Second iteration, in topological order, to create stages and tasks
        let mut stages = Vec::with_capacity(included.len());
        let mut stage: PlanStage = PlanStage::default();
        for resource in &self.topological_order {
            // Only include resources included above
            if !included.contains(resource) {
                continue;
            }

            // Get the `Code` resource to be executed
            let code = match resource {
                Resource::Code(code) => code,
                _ => continue,
            };

            // Only execute resources for which there is a kernel capable of executing code
            let selector = KernelSelector::new(None, code.language.clone(), None);
            let kernel = selector.select(&kernels);
            let (kernel_name, kernel_forkable) = match kernel {
                Some(kernel) => (Some(kernel.name.clone()), kernel.forkable),
                None => continue,
            };

            let resource_info = self.get_resource_info(resource)?;

            // Determine if the taks should run in a fork
            let is_fork = Self::should_run_in_fork(
                kernel_forkable,
                resource_info,
                stage.tasks.len(),
                &options,
            );

            // Create the task
            let task = PlanTask {
                resource_info: resource_info.clone(),
                kernel_name,
                is_fork,
            };

            // Add the task to a new, or the current, stage
            if Self::needs_new_stage(&stage.tasks) {
                if !stage.tasks.is_empty() {
                    stages.push(stage);
                    stage = PlanStage::default();
                }
                stages.push(PlanStage { tasks: vec![task] });
            } else {
                stage.tasks.push(task);
            }
        }

        // Collect any tasks not yet added (e.g. a `CodeExpression` at end of document)
        if !stage.tasks.is_empty() {
            stages.push(stage);
        }

        Ok(Plan {
            options: PlanOptions {
                ordering: PlanOrdering::Topological,
                ..options
            },
            stages,
        })
    }

    /// Convert the graph to some format
    pub fn to_format(&self, format: &str) -> Result<String> {
        Ok(match format {
            "dot" => self.to_dot(),
            "json" => serde_json::to_string_pretty(self)?,
            "yaml" => serde_yaml::to_string(self)?,
            _ => bail!("Unknown graph format '{}'", format),
        })
    }

    /// Convert the graph to a visualization nodes and edges
    pub fn to_dot(&self) -> String {
        let nodes = self
            .indices
            .iter()
            .map(|(resource, node)| {
                let index = node.index();

                let path = match resource {
                    Resource::Symbol(symbol) => symbol.path.clone(),
                    Resource::Node(node) => node.path.clone(),
                    Resource::File(file) => file.path.clone(),
                    _ => PathBuf::new(),
                };
                let path = path
                    .strip_prefix(&self.path)
                    .unwrap_or(&path)
                    .to_slash_lossy()
                    .into_owned();

                let (shape, fill_color, label) = match resource {
                    Resource::Symbol(symbol) => (
                        "diamond",
                        "#efb8b8",
                        format!(
                            "{}{}",
                            if !symbol.kind.is_empty() {
                                format!("{}\\n", symbol.kind)
                            } else {
                                "".to_string()
                            },
                            symbol.name
                        ),
                    ),
                    Resource::Code(code) => {
                        let label = if let Some(lang) = &code.language {
                            format!("{} {}\\n{}", lang, code.kind, code.id)
                        } else {
                            format!("{}\\n{}", code.kind, code.id)
                        };
                        ("box", "#efe0a6", label)
                    }
                    Resource::Node(node) => {
                        let label = format!("{}\\n{}", node.kind, node.id);
                        ("box", "#efe0b8", label)
                    }
                    Resource::File(..) => ("note", "#d1efb8", path.clone()),
                    Resource::Module(module) => ("invhouse", "#b8efed", module.name.clone()),
                    Resource::Url(url) => ("box", "#cab8ef", url.url.clone()),
                };

                let node = match resource {
                    Resource::File(..) => format!(
                        r#"  n{index} [shape="point", style="invis" label="{label}"]"#,
                        index = index,
                        label = label
                    ),
                    _ => format!(
                        r#"  n{index} [shape="{shape}" fillcolor="{fill_color}" label="{label}"]"#,
                        index = index,
                        shape = shape,
                        fill_color = fill_color,
                        label = label.replace('\"', "\\\"")
                    ),
                };

                (path, node)
            })
            .collect::<Vec<(String, String)>>();

        let mut clusters: HashMap<String, Vec<String>> = HashMap::new();
        for (path, node) in nodes {
            clusters.entry(path).or_default().push(node)
        }

        let path_to_cluster = |path: &Path| {
            clusters
                .keys()
                .position(|key| {
                    key == &path
                        .strip_prefix(&self.path)
                        .unwrap_or(path)
                        .to_slash_lossy()
                })
                .unwrap_or(0)
        };

        let subgraphs = clusters
            .iter()
            .enumerate()
            .map(|(index, (label, nodes))| {
                if label.is_empty() {
                    nodes.join("\n")
                } else {
                    [
                        &format!("  subgraph cluster{} {{\n", index),
                        &format!("    label=\"{}\" fillcolor=\"#d1efb8\"\n  ", label),
                        &nodes.join("\n  "),
                        "\n  }",
                    ]
                    .concat()
                }
            })
            .collect::<Vec<String>>()
            .join("\n");

        let edges = self
            .graph
            .edge_indices()
            .filter_map(|edge| {
                if let (Some((from, to)), Some(relation)) = (
                    self.graph.edge_endpoints(edge),
                    self.graph.edge_weight(edge),
                ) {
                    let (label, style) = match relation {
                        Relation::Converts(relations::Converts { auto: active }) => (
                            relation.to_string(),
                            if *active { "solid" } else { "dashed" },
                        ),
                        Relation::Declares(relations::Declares { range })
                        | Relation::Assigns(relations::Assigns { range })
                        | Relation::Uses(relations::Uses { range })
                        | Relation::Imports(relations::Imports { range })
                        | Relation::Reads(relations::Reads { range })
                        | Relation::Writes(relations::Writes { range }) => {
                            let label = if *range == relations::NULL_RANGE {
                                relation.to_string()
                            } else {
                                format!("{} (L{})", relation, range.0 + 1)
                            };
                            (label, "solid")
                        }
                        _ => (relation.to_string(), "solid"),
                    };

                    let ltail = if let Some(Resource::File(file)) = self.graph.node_weight(from) {
                        format!(" ltail=\"cluster{}\"", path_to_cluster(&file.path))
                    } else {
                        "".to_string()
                    };

                    let lhead = if let Some(Resource::File(file)) = self.graph.node_weight(to) {
                        format!(" lhead=\"cluster{}\"", path_to_cluster(&file.path))
                    } else {
                        "".to_string()
                    };

                    Some(format!(
                        r#"  n{from} -> n{to} [label="{label}" style="{style}"{ltail}{lhead}]"#,
                        from = from.index(),
                        to = to.index(),
                        label = label,
                        style = style,
                        ltail = ltail,
                        lhead = lhead,
                    ))
                } else {
                    None
                }
            })
            .collect::<Vec<String>>()
            .join("\n");

        format!(
            r#"digraph {{
  graph [rankdir=LR compound=true fontname=Helvetica fontsize=12 labeljust=l color=gray]
  node [style=filled fontname=Helvetica fontsize=11]
  edge [fontname=Helvetica fontsize=10]

{subgraphs}

{edges}
}}
"#,
            subgraphs = subgraphs,
            edges = edges
        )
    }
}

#[derive(Debug, Display, JsonSchema, Serialize)]
#[serde(rename_all = "lowercase", crate = "common::serde")]
#[strum(serialize_all = "lowercase")]
pub enum GraphEventType {
    Updated,
}

#[skip_serializing_none]
#[derive(Debug, JsonSchema, Serialize)]
#[serde(crate = "common::serde")]
#[schemars(deny_unknown_fields)]
pub struct GraphEvent {
    /// The path of the project (absolute)
    project: PathBuf,

    /// The type of event
    #[serde(rename = "type")]
    type_: GraphEventType,

    /// The graph at the time of the event
    #[schemars(schema_with = "GraphEvent::graph_schema")]
    graph: Graph,
}

impl GraphEvent {
    /// Generate the JSON Schema for the `graph` property
    fn graph_schema(_generator: &mut SchemaGenerator) -> Schema {
        Schema::Object(SchemaObject {
            reference: some_string!("Graph"),
            ..Default::default()
        })
    }

    /// Publish a `GraphEvent`.
    ///
    /// Will publish an event under the `projects:<project>:graph` topic.
    pub fn publish(project: &Path, type_: GraphEventType, graph: &Graph) {
        let topic = &format!("projects:{}:graph", project.display());
        let event = GraphEvent {
            project: project.to_path_buf(),
            type_,
            graph: graph.clone(),
        };
        events::publish(topic, &event)
    }
}

/// Get JSON Schemas for this crate
pub fn schemas() -> Result<serde_json::Value> {
    Ok(json!([
        schema_for!(Resource),
        schema_for!(Relation),
        schema_for!(GraphEvent),
        serde_json::json!({
            "$id": "Triple",
            "title": "Triple",
            "description": "A subject-relation-object triple",
            "type" : "array",
            "items": [
                {
                    "tsType": "Resource"
                },
                {
                    "tsType": "Relation"
                },
                {
                    "tsType": "Resource"
                }
            ],
            "minItems": 3,
            "maxItems": 3
        }),
        serde_json::json!({
            "$id": "Graph",
            "title": "Graph",
            "description": "A project dependency graph",
            "type" : "object",
            "required": ["nodes", "edges"],
            "properties": {
                "nodes": {
                    "description": "The resources in the graph",
                    "type": "array",
                    "items": {
                        "tsType": "Resource"
                    },
                    "isRequired": true
                },
                "edges": {
                    "description": "The relations between resources in the graph",
                    "type": "array",
                    "items": {
                        "type": "object",
                        "required": ["from", "to", "relation"],
                        "properties": {
                            "from": "integer",
                            "to": "integer",
                            "relation" : {
                                "tsType": "Resource"
                            }
                        },
                        "additionalProperties": false
                    },
                    "isRequired": true
                }
            },
            "additionalProperties": false
        }),
    ]))
}
