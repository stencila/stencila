use eyre::{bail, Result};
use graph_triples::{direction, relations, Direction, Pairs, Relation, Resource, Triple};
use path_slash::PathExt;
use petgraph::{
    graph::NodeIndex,
    stable_graph::StableGraph,
    visit::{EdgeRef, IntoEdgeReferences, IntoNodeReferences},
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
    collections::HashMap,
    path::{Path, PathBuf},
};
use strum::Display;
use utils::some_string;

/// A dependency graph for a project or document
#[derive(Debug, Default, Clone)]
pub struct Graph {
    /// The path of the project or document that this graph is for
    ///
    /// Primarily used to make file paths relative in visualizations and
    /// if ever persisting the graph.
    path: PathBuf,

    /// The graph itself
    ///
    /// Use a `petgraph::StableGraph` so that nodes can be added and removed
    /// without changing node indices.
    graph: StableGraph<Resource, Relation>,

    /// Indices of the nodes in the tree
    ///
    /// This is necessary to keep track of which resources
    /// are already in the graph and re-use their index if they are.
    indices: HashMap<Resource, NodeIndex>,
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
            indices: HashMap::new(),
            graph: StableGraph::new(),
        }
    }

    /// Create a graph from set of dependency relations
    pub fn from_relations<P: AsRef<Path>>(path: P, relations: &[(Resource, Pairs)]) -> Graph {
        let mut graph = Graph::new(path);
        for (subject, pairs) in relations {
            for (relation, object) in pairs {
                graph.add_triple((subject.clone(), relation.clone(), object.clone()));
            }
        }
        graph
    }

    /// Add a resource to the graph
    pub fn add_resource(&mut self, resource: Resource) {
        if self.indices.get(&resource).is_none() {
            let index = self.graph.add_node(resource.clone());
            self.indices.insert(resource, index);
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
                    Resource::Node(node) => {
                        let label = if !node.id.starts_with('_') {
                            format!("{}\\n{}", node.kind, node.id)
                        } else {
                            node.kind.clone()
                        };
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
