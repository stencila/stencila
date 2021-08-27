use crate::{pubsub::publish, utils::schemas};
use derivative::Derivative;
use eyre::Result;
use path_slash::PathExt;
use petgraph::{
    graph::NodeIndex,
    stable_graph::StableGraph,
    visit::{EdgeRef, IntoEdgeReferences, IntoNodeReferences},
};
use schemars::{gen::SchemaGenerator, schema::Schema, JsonSchema};
use serde::{ser::SerializeMap, Serialize};
use serde_json::json;
use serde_with::skip_serializing_none;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use strum::{Display, ToString};

/// A resource in a dependency graph (the nodes of the graph)
#[derive(Debug, Clone, PartialEq, Eq, Hash, JsonSchema, Serialize)]
#[serde(tag = "type")]
pub enum Resource {
    /// A symbol within code, within a project file
    Symbol(resources::Symbol),

    /// A node within a project file
    Node(resources::Node),

    /// A file within the project
    File(resources::File),

    /// A declared project `Source`
    Source(resources::Source),

    /// A programming language module, usually part of an external package
    Module(resources::Module),

    /// A URL to a remote resource
    Url(resources::Url),
}

pub mod resources {
    use super::*;
    use std::path::{Path, PathBuf};

    #[derive(Debug, Clone, Derivative, JsonSchema, Serialize)]
    #[derivative(PartialEq, Eq, Hash)]
    #[schemars(deny_unknown_fields)]
    pub struct Symbol {
        /// The path of the file that the symbol is defined in
        #[serde(serialize_with = "serialize_path")]
        pub path: PathBuf,

        /// The name/identifier of the symbol
        pub name: String,

        /// The type of the object that the symbol refers to (e.g `Number`, `Function`)
        ///
        /// Should be used as a hint only, and as such is excluded from
        /// equality and hash functions.
        #[derivative(PartialEq = "ignore")]
        #[derivative(Hash = "ignore")]
        pub kind: String,
    }

    /// Create a new `Symbol` resource
    pub fn symbol(path: &Path, name: &str, kind: &str) -> Resource {
        Resource::Symbol(Symbol {
            path: path.to_path_buf(),
            name: name.into(),
            kind: kind.into(),
        })
    }

    #[derive(Debug, Clone, PartialEq, Eq, Hash, JsonSchema, Serialize)]
    #[schemars(deny_unknown_fields)]
    pub struct Node {
        /// The path of the file that the node is defined in
        #[serde(serialize_with = "serialize_path")]
        pub path: PathBuf,

        /// The id of the node with the document
        pub id: String,

        /// The type of node e.g. `Parameter`, `CodeChunk`
        pub kind: String,
    }

    /// Create a new `Node` resource
    pub fn node(path: &Path, id: &str, kind: &str) -> Resource {
        Resource::Node(Node {
            path: path.to_path_buf(),
            id: id.into(),
            kind: kind.into(),
        })
    }

    #[derive(Debug, Clone, PartialEq, Eq, Hash, JsonSchema, Serialize)]
    #[schemars(deny_unknown_fields)]
    pub struct File {
        /// The path of the file
        #[serde(serialize_with = "serialize_path")]
        pub path: PathBuf,
    }

    /// Create a new `File` resource
    pub fn file(path: &Path) -> Resource {
        Resource::File(File {
            path: path.to_path_buf(),
        })
    }

    #[derive(Debug, Clone, PartialEq, Eq, Hash, JsonSchema, Serialize)]
    #[schemars(deny_unknown_fields)]
    pub struct Source {
        /// The name of the project source
        pub name: String,
    }

    /// Create a new `Source` resource
    pub fn source(name: &str) -> Resource {
        Resource::Source(Source { name: name.into() })
    }

    #[derive(Debug, Clone, PartialEq, Eq, Hash, JsonSchema, Serialize)]
    #[schemars(deny_unknown_fields)]
    pub struct Module {
        /// The programming language of the module
        pub language: String,

        /// The name of the module
        pub name: String,
    }

    /// Create a new `Module` resource
    pub fn module(language: &str, name: &str) -> Resource {
        Resource::Module(Module {
            language: language.into(),
            name: name.into(),
        })
    }

    #[derive(Debug, Clone, Default, PartialEq, Eq, Hash, JsonSchema, Serialize)]
    #[schemars(deny_unknown_fields)]
    pub struct Url {
        /// The URL of the external resource
        pub url: String,
    }

    /// Create a new `Url` resource
    pub fn url(url: &str) -> Resource {
        Resource::Url(Url { url: url.into() })
    }
}

/// Serialize the `path` fields of resources so that they use Unix forward slash
/// separators on all platforms.
fn serialize_path<S>(path: &Path, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    path.to_slash_lossy().serialize(serializer)
}

/// The relation between two resources in a dependency graph (the edges of the graph)
///
/// Some relations carry additional information such whether the relation is active
/// (`Import` and `Convert`) or the range that they occur in code (`Assign`, `Use`, `Read`) etc
#[derive(Debug, Display, Clone, JsonSchema, Serialize)]
#[serde(tag = "type")]
pub enum Relation {
    Assign(relations::Assign),
    Convert(relations::Convert),
    Embed,
    Import(relations::Import),
    Include,
    Link,
    Read(relations::Read),
    Use(relations::Use),
    Write(relations::Write),
}

/// The two dimensional range that a relation is defined within some
/// code (line start, column start, line end, column end).
pub type Range = (usize, usize, usize, usize);

/// A null range which can be used in places where we do not know where
/// in the `subject` the relation is defined.
pub const NULL_RANGE: Range = (0, 0, 0, 0);

pub mod relations {
    use super::*;

    /// Assigns a symbol
    #[derive(Debug, Clone, JsonSchema, Serialize)]
    #[schemars(deny_unknown_fields)]
    pub struct Assign {
        /// The range within code that the assignment is done
        pub range: Range,
    }

    /// Create a new `Assign` relation
    pub fn assigns(range: Range) -> Relation {
        Relation::Assign(Assign { range })
    }

    /// Imports a file from a `Source`
    #[derive(Debug, Clone, JsonSchema, Serialize)]
    #[schemars(deny_unknown_fields)]
    pub struct Import {
        /// Whether or not the import is automatically updated
        pub auto: bool,
    }

    /// Create a new `Import` relation
    pub fn imports(auto: bool) -> Relation {
        Relation::Import(Import { auto })
    }

    /// Converts a file into another
    #[derive(Debug, Clone, JsonSchema, Serialize)]
    #[schemars(deny_unknown_fields)]
    pub struct Convert {
        /// Whether or not the conversion is automatically updated
        pub auto: bool,
    }

    /// Create a new `Convert` relation
    pub fn converts(auto: bool) -> Relation {
        Relation::Convert(Convert { auto })
    }

    /// Reads from a file
    #[derive(Debug, Clone, JsonSchema, Serialize)]
    #[schemars(deny_unknown_fields)]
    pub struct Read {
        /// The range within code that the read is declared
        pub range: Range,
    }

    /// Create a new `Read` relation
    pub fn reads(range: Range) -> Relation {
        Relation::Read(Read { range })
    }

    /// Uses a symbol or module
    #[derive(Debug, Clone, JsonSchema, Serialize)]
    #[schemars(deny_unknown_fields)]
    pub struct Use {
        /// The range within code that the use is declared
        pub range: Range,
    }

    /// Create a new `Use` relation
    pub fn uses(range: Range) -> Relation {
        Relation::Use(Use { range })
    }

    /// Writes to a file
    #[derive(Debug, Clone, JsonSchema, Serialize)]
    #[schemars(deny_unknown_fields)]
    pub struct Write {
        /// The range within code that the write is declared
        pub range: Range,
    }

    /// Create a new `Write` relation
    pub fn writes(range: Range) -> Relation {
        Relation::Write(Write { range })
    }
}

/// The direction to represent the flow of information from subject to object
pub enum Direction {
    From,
    To,
}

/// Get the the `Direction` for a `Relation`
pub fn direction(relation: &Relation) -> Direction {
    match relation {
        Relation::Assign(..) => Direction::To,
        Relation::Convert(..) => Direction::To,
        Relation::Embed => Direction::From,
        Relation::Import(..) => Direction::To,
        Relation::Include => Direction::From,
        Relation::Link => Direction::To,
        Relation::Read(..) => Direction::From,
        Relation::Use(..) => Direction::From,
        Relation::Write(..) => Direction::To,
    }
}

/// A subject-relation-object triple
pub type Triple = (Resource, Relation, Resource);

/// A project dependency graph
#[derive(Debug, Default, Clone)]
pub struct Graph {
    /// The path of the project that this graph is for
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

impl Graph {
    /// Create a new graph
    pub fn new(path: PathBuf) -> Graph {
        Graph {
            path,
            indices: HashMap::new(),
            graph: StableGraph::new(),
        }
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
                            let label = if *range == NULL_RANGE {
                                relation.to_string()
                            } else {
                                format!("{} L{}", relation, range.0 + 1)
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

#[derive(Debug, JsonSchema, Serialize, ToString)]
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
    #[schemars(schema_with = "GraphEvent::schema_graph")]
    graph: Graph,
}

impl GraphEvent {
    /// Generate the JSON Schema for the `graph` property to avoid nesting
    fn schema_graph(_generator: &mut SchemaGenerator) -> Schema {
        schemas::typescript("Graph", true)
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
        publish(topic, &event)
    }
}

/// Get JSON Schemas for this modules
pub fn schemas() -> Result<serde_json::Value> {
    let schemas = serde_json::Value::Array(vec![
        schemas::generate::<Resource>()?,
        schemas::generate::<Relation>()?,
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
        schemas::generate::<GraphEvent>()?,
    ]);
    Ok(schemas)
}
