use eyre::{bail, eyre, Result};
use graph_triples::{
    direction, relations, resources::ResourceDigest, Direction, Pairs, Relation, Resource,
    ResourceInfo, Triple,
};
use hash_utils::{sha2, sha2::Digest};
use kernels::{Kernel, KernelSelector};
use path_slash::PathExt;
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
use serde::{ser::SerializeMap, Serialize};
use serde_json::json;
use serde_with::skip_serializing_none;
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    path::{Path, PathBuf},
};
use strum::Display;
use utils::some_string;

use crate::{Plan, PlanOptions, PlanOrdering, Stage, Step};

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

                // Add info from `resources`
                if let Some(resource_info) = self.resources.get(resource) {
                    if let Some(pure) = resource_info.pure {
                        obj.insert("pure".to_string(), json!(pure));
                    }
                    if let Some(dependencies) = &resource_info.dependencies {
                        obj.insert("dependencies".to_string(), json!(dependencies.len()));
                    }
                    if let Some(depth) = resource_info.depth {
                        obj.insert("depth".to_string(), json!(depth));
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

    /// Create a graph from a set of [`ResourceInfo`] objects
    pub fn from_resource_infos<P: AsRef<Path>>(
        path: P,
        resource_infos: Vec<ResourceInfo>,
    ) -> Result<Self> {
        let mut graph = Graph::new(path);
        graph.add_resource_infos(resource_infos);
        graph.update(None)?;
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

    /// Add a set of [`ResourceInfo`] objects to the graph
    pub fn add_resource_infos(&mut self, resource_infos: Vec<ResourceInfo>) {
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
                for (relation, object) in relations.into_iter() {
                    // Add the object resource (if it is not already)
                    let object_index = self.add_resource(object, None);

                    // Add an edge
                    let (from, to) = match direction(&relation) {
                        Direction::From => (object_index, subject_index),
                        Direction::To => (subject_index, object_index),
                    };
                    self.graph.add_edge(from, to, relation);
                }
            }
        }
    }

    /// Get [`ResourceInfo`] objects in the graph
    pub fn get_resource_infos(&mut self) -> Vec<&ResourceInfo> {
        self.resources.values().collect()
    }

    /// Update the graph, usually in response to a change in one of it's resources
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
            let mut dependencies_digest = sha2::Sha256::new();
            let mut dependencies_unsynced = 0;
            for incoming_index in incomings {
                let dependency = &graph[incoming_index];
                let dependency_info = self
                    .resources
                    .get(dependency)
                    .ok_or_else(|| eyre!("No info for dependency"))?;

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

                // Update the digest of dependencies using a concatenation of their content, semantic,
                // and dependencies digests (note that some of these may be empty which is why we
                // need to include all three).
                let digest = [
                    compile_digest.content_digest.as_str(),
                    compile_digest.semantic_digest.as_str(),
                    compile_digest.dependencies_digest.as_str(),
                ]
                .concat();
                dependencies_digest.update(digest);

                // Update the number of `Code` dependencies that are unsynced. This is transitive,
                // so if the resource is not code (e.g. a `Symbol`) then add its value.
                if matches!(dependency, Resource::Code(..)) {
                    if let Some(execute_digest) = &dependency_info.execute_digest {
                        // Has dependency changed since it was last executed?
                        if execute_digest != compile_digest {
                            dependencies_unsynced += 1;
                        }
                    } else {
                        // Dependency has not been executed
                        dependencies_unsynced += 1;
                    }
                } else {
                    dependencies_unsynced += compile_digest.dependencies_unsynced;
                }
            }

            // If there are no dependencies then `dependencies_digest` is an empty string
            let dependencies_digest = match !dependencies.is_empty() {
                true => ResourceDigest::base64_encode(&dependencies_digest.finalize()),
                false => String::default(),
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
                    compile_digest.dependencies_unsynced = dependencies_unsynced;
                }
                None => {
                    let mut compile_digest = resource.digest();
                    compile_digest.dependencies_digest = dependencies_digest;
                    compile_digest.dependencies_unsynced = dependencies_unsynced;
                    resource_info.compile_digest = Some(compile_digest);
                }
            }
        }

        Ok(())
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
            None => kernels::available().await?,
        };

        let options = options.unwrap_or_default();
        match options.ordering {
            PlanOrdering::Appearance => self.plan_appearance(start, kernels, options),
            PlanOrdering::Topological => self.plan_topological(start, kernels, options),
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
    pub fn plan_appearance(
        &self,
        start: Option<Resource>,
        kernels: Vec<Kernel>,
        options: PlanOptions,
    ) -> Result<Plan> {
        let mut stages: Vec<Stage> = Vec::with_capacity(self.appearance_order.len());
        let mut stage: Stage = Stage::default();
        let mut started = start.is_none();
        for resource in &self.appearance_order {
            // Should we start collecting steps?
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

            let resource_info = self
                .resources
                .get(resource)
                .ok_or_else(|| eyre!("No info for resource"))?;

            // If (a) the kernel is forkable, (b) the code is `@pure` (inferred or declared),
            // and (c) the maximum concurrency has not been exceeded then execute the step in a fork
            let is_fork = kernel_forkable
                && resource_info.is_pure()
                && stage.steps.len() < options.max_concurrency;

            // Create the step and add it to the current stage
            let step = Step {
                resource_info: resource_info.clone(),
                kernel_name,
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
    /// - `start`: The node at which the plan should start. Only nodes that have `start`
    ///            as a dependency (direct or transitive) will be executed. If `None` then
    ///            the plan applies to all nodes in the document.
    ///
    /// - `options`: Options for the plan
    pub fn plan_topological(
        &self,
        start: Option<Resource>,
        kernels: Vec<Kernel>,
        options: PlanOptions,
    ) -> Result<Plan> {
        // First iteration, in topological order, to determine which resources to include
        let mut include = HashSet::new();
        let mut started = start.is_none();
        for resource in &self.topological_order {
            // Should we start collecting steps?
            if !started {
                if let Some(start) = &start {
                    started = start == resource;
                }
            }
            if !started {
                continue;
            }

            let resource_info = self
                .resources
                .get(resource)
                .ok_or_else(|| eyre!("No info for resource"))?;

            // Only include resources that are `start` or have `start` in their dependencies
            if let Some(start) = &start {
                if !(start == resource
                    || resource_info
                        .dependencies
                        .as_ref()
                        .map_or(false, |deps| deps.contains(start)))
                {
                    continue;
                }
            }

            // Only include `Code` resources (i.e. ignore `Symbol`s etc which will also be in the dependency
            // graph and therefore in `topological_order` as well)
            match resource {
                Resource::Code(..) => include.insert(resource),
                _ => continue,
            };

            /*
            TODO
            // Include any dependencies that are not yet included and which have not been
            // executed yet or have a change in digest.
            for dependency in resource_info.dependencies {
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

            */
        }

        // Second iteration, in topological order, to create stages and steps
        let mut stages = Vec::with_capacity(include.len());
        let mut stage: Stage = Stage::default();
        for resource in &self.topological_order {
            // Only include resources included above
            if !include.contains(resource) {
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

            let resource_info = self
                .resources
                .get(resource)
                .ok_or_else(|| eyre!("No info for resource"))?;

            // If (a) the kernel is forkable, (b) the code is `@pure` (inferred or declared),
            // and (c) the maximum concurrency has not been exceeded then execute the step in a fork
            let is_fork = kernel_forkable
                && resource_info.is_pure()
                && stage.steps.len() < options.max_concurrency;

            // Create the step and add it to the current stage
            let step = Step {
                resource_info: resource_info.clone(),
                kernel_name,
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
                    .to_slash_lossy();

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
                    Resource::Source(source) => ("house", "#efb8d4", source.name.clone()),
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
                        Relation::Convert(relations::Convert { auto: active })
                        | Relation::Import(relations::Import { auto: active }) => (
                            relation.to_string(),
                            if *active { "solid" } else { "dashed" },
                        ),
                        Relation::Assign(relations::Assign { range })
                        | Relation::Use(relations::Use { range })
                        | Relation::Read(relations::Read { range })
                        | Relation::Write(relations::Write { range }) => {
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
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum GraphEventType {
    Updated,
}

#[skip_serializing_none]
#[derive(Debug, JsonSchema, Serialize)]
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
