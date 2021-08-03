use crate::utils::schemas;
use eyre::Result;
use petgraph::{graph::NodeIndex, stable_graph::StableGraph};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use strum::{Display, EnumString};

/// A resource in a dependency graph (the nodes of the graph)
#[derive(Debug, Clone, PartialEq, Eq, Hash, EnumString, JsonSchema, Serialize, Deserialize)]
#[serde(tag = "type", content = "id")]
#[schemars(deny_unknown_fields)]
pub enum Resource {
    // Within-code resources
    Symbol(String),
    Variable(String),
    Function(String),

    // Within-document resources
    // Store the relative path (within project) and address (within document)
    // of the resource.
    Include(String),
    Link(String),
    Embed(String),
    CodeChunk(String),
    CodeExpression(String),

    // Within-project resources
    // Store the relative path (within project) of the resource
    File(String),
    SoftwareSourceCode(String),
    AudioObject(String),
    ImageObject(String),
    VideoObject(String),

    // External resources
    // Store unique identifier for resource
    Module(String),
    Url(String),
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
                    Resource::Symbol(id) => ("diamond", "#adebbc", id.as_str()),
                    Resource::Variable(id) => ("diamond", "#adebbc", id.as_str()),
                    Resource::Function(id) => ("diamond", "#adebbc", id.as_str()),

                    Resource::Include(id) => ("diamond", "#adebbc", id.as_str()),
                    Resource::Link(id) => ("diamond", "#adebbc", id.as_str()),
                    Resource::Embed(id) => ("house", "#adebbc", id.as_str()),
                    Resource::CodeChunk(id) => ("parallelogram", "#adebbc", id.as_str()),
                    Resource::CodeExpression(id) => ("parallelogram", "#d6ebad", id.as_str()),

                    Resource::File(id) => ("note", "#adc8eb", id.as_str()),
                    Resource::SoftwareSourceCode(id) => ("box", "#adebbc", id.as_str()),
                    Resource::AudioObject(id) => ("box", "#adebbc", id.as_str()),
                    Resource::ImageObject(id) => ("box", "#adebbc", id.as_str()),
                    Resource::VideoObject(id) => ("box", "#adebbc", id.as_str()),

                    Resource::Module(id) => ("invhouse", "#adebbc", id.as_str()),
                    Resource::Url(id) => ("box", "#adebbc", id.as_str()),
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
