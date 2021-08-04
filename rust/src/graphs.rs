use crate::utils::schemas;
use derivative::Derivative;
use eyre::Result;
use petgraph::{graph::NodeIndex, stable_graph::StableGraph};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use strum::{Display, EnumString};

/// A resource in a dependency graph (the nodes of the graph)
#[derive(Debug, Clone, PartialEq, Eq, Hash, EnumString, JsonSchema, Serialize)]
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
    #[derive(Debug, Clone, Default, Derivative, JsonSchema, Serialize)]
    #[derivative(PartialEq, Eq, Hash)]
    #[schemars(deny_unknown_fields)]
    pub struct Symbol {
        /// The path of the file that the symbol is defined in
        pub path: String,

        /// The name/identifier of the symbol
        pub name: String,

        /// The kind of the object that the symbol refers to
        ///
        /// Should be used as a hint only, and as such is excluded from
        /// equality and hash functions.
        #[derivative(PartialEq = "ignore")]
        #[derivative(Hash = "ignore")]
        pub kind: String,
    }

    impl Symbol {
        pub fn label(&self) -> String {
            [&self.path, "@", &self.name].concat()
        }
    }

    /// Create a new `Symbol` resource
    pub fn symbol(path: &str, name: &str, kind: &str) -> Resource {
        Resource::Symbol(Symbol {
            path: path.into(),
            name: name.into(),
            kind: kind.into(),
        })
    }

    #[derive(Debug, Clone, Default, PartialEq, Eq, Hash, JsonSchema, Serialize)]
    #[schemars(deny_unknown_fields)]
    pub struct Node {
        /// The path of the file that the node is defined in
        pub path: String,

        /// The address of the node
        pub address: String,

        /// The type of node e.g. "CodeChunk"
        pub kind: String,
    }

    impl Node {
        pub fn label(&self) -> String {
            [&self.path, "&", &self.address].concat()
        }
    }

    /// Create a new `Symbol` resource
    pub fn node(path: &str, address: &str, kind: &str) -> Resource {
        Resource::Node(Node {
            path: path.into(),
            address: address.into(),
            kind: kind.into(),
        })
    }

    #[derive(Debug, Clone, Default, PartialEq, Eq, Hash, JsonSchema, Serialize)]
    #[schemars(deny_unknown_fields)]
    pub struct File {
        /// The path of the file
        pub path: String,
    }

    impl File {
        pub fn label(&self) -> String {
            self.path.clone()
        }
    }

    /// Create a new `File` resource
    pub fn file(path: &str) -> Resource {
        Resource::File(File { path: path.into() })
    }

    #[derive(Debug, Clone, Default, PartialEq, Eq, Hash, JsonSchema, Serialize)]
    #[schemars(deny_unknown_fields)]
    pub struct Source {
        /// The name of the project source
        pub name: String,
    }

    impl Source {
        pub fn label(&self) -> String {
            self.name.clone()
        }
    }

    /// Create a new `Source` resource
    pub fn source(name: &str) -> Resource {
        Resource::Source(Source { name: name.into() })
    }

    #[derive(Debug, Clone, Default, PartialEq, Eq, Hash, JsonSchema, Serialize)]
    #[schemars(deny_unknown_fields)]
    pub struct Module {
        /// The programming language of the module
        pub language: String,

        /// The name of the module
        pub name: String,
    }

    impl Module {
        pub fn label(&self) -> String {
            [&self.language, ":", &self.name].concat()
        }
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

    impl Url {
        pub fn label(&self) -> String {
            self.url.clone()
        }
    }

    /// Create a new `Url` resource
    pub fn url(url: &str) -> Resource {
        Resource::Url(Url { url: url.into() })
    }
}

/// The relation between two resources in a dependency graph (the edges of the graph)
#[derive(
    Debug, Display, Clone, PartialEq, Eq, Hash, EnumString, JsonSchema, Serialize, Deserialize,
)]
#[serde(rename_all = "camelCase")]
pub enum Relation {
    Assigns,
    Embeds,
    Imports,
    Includes,
    Links,
    Reads,
    Uses,
    Writes,
}

/// The direction to represent the flow of information from subject to object.
pub enum Direction {
    From,
    To,
}

/// Get the the `Direction` for a `Relation`
pub fn direction(relation: &Relation) -> Direction {
    match relation {
        Relation::Assigns => Direction::To,
        Relation::Embeds => Direction::From,
        Relation::Imports => Direction::From,
        Relation::Includes => Direction::From,
        Relation::Links => Direction::To,
        Relation::Reads => Direction::From,
        Relation::Uses => Direction::From,
        Relation::Writes => Direction::To,
    }
}

/// A subject-relation-object triple
pub type Triple = (Resource, Relation, Resource);

/// A project dependency graph
#[derive(Debug, Default, Clone, Serialize)]
pub struct Graph {
    /// The graph itself
    ///
    /// Use a `petgraph::StableGraph` so that nodes can be added and removed
    /// without changing node indices.
    #[serde(flatten)]
    graph: StableGraph<Resource, Relation>,

    /// Indices of the nodes in the tree
    ///
    /// This is necessary to keep track of which resources
    /// are already in the graph and re-use their index if they are.
    #[serde(skip)]
    indices: HashMap<Resource, NodeIndex>,
}

impl Graph {
    /// Create a new graph
    pub fn new() -> Graph {
        Graph {
            indices: HashMap::new(),
            graph: StableGraph::new(),
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

    /// Convert the graph to a visualization nodes and edges
    pub fn to_dot(&self) -> String {
        let nodes = self
            .indices
            .iter()
            .map(|(resource, node)| {
                let (shape, fill_color, label) = match resource {
                    Resource::Symbol(resource) => ("diamond", "#efb8b8", resource.label()),
                    Resource::Node(resource) => ("house", "#efe0b8", resource.label()),
                    Resource::File(resource) => ("note", "#d1efb8", resource.label()),
                    Resource::Source(resource) => ("ellipse", "#efb8d4", resource.label()),
                    Resource::Module(resource) => ("invhouse", "#b8efed", resource.label()),
                    Resource::Url(resource) => ("box", "#cab8ef", resource.label()),
                };

                format!(
                    r#"  n{id} [shape="{shape}" fillcolor="{fill_color}" label="{label}"]"#,
                    id = node.index(),
                    shape = shape,
                    fill_color = fill_color,
                    label = label.replace('\"', "\\\"")
                )
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
                    Some(format!(
                        r#"  n{from} -> n{to} [label="{label}"]"#,
                        from = from.index(),
                        to = to.index(),
                        label = relation.to_string().replace('\"', "\\\"")
                    ))
                } else {
                    None
                }
            })
            .collect::<Vec<String>>()
            .join("\n");

        format!(
            r#"digraph {{
  node [style="filled" fontname=Helvetica fontsize=11]
  edge [fontname=Helvetica fontsize=10]

{nodes}

{edges}
}}
"#,
            nodes = nodes,
            edges = edges
        )
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
                        "tsType": "Resource",
                        "isRequired": true
                    }
                },
                "edges": {
                    "description": "The relations between resources in the graph",
                    "type": "array",
                    "items": {
                        "type": "array",
                        "items": [
                            {
                                "type": "integer"
                            },
                            {
                                "type": "integer"
                            },
                            {
                                "tsType": "Relation"
                            }
                        ],
                        "minItems": 3,
                        "maxItems": 3
                    }
                }
            },
            "additionalProperties": false
        }),
    ]);
    Ok(schemas)
}
