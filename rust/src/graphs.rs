use petgraph::{graph::NodeIndex, stable_graph::StableGraph};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use strum::{Display, EnumString};

#[derive(Debug, Clone, PartialEq, Eq, Hash, EnumString, JsonSchema, Serialize, Deserialize)]
#[serde(tag = "type", content = "id")]
pub enum Resource {
    // Within-code resources
    Symbol(String),
    Variable(String),
    Function(String),

    // Within-document resources
    // Store the relative path (within project) and address (within document)
    // of the resource.
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

pub struct ResourceVisualization {
    id: String,
    label: String,
    shape: String,
    fill_color: String,
}

impl ResourceVisualization {
    pub fn new(index: usize, resource: &Resource) -> ResourceVisualization {
        let (shape, fill_color, label) = match resource {
            Resource::Symbol(id) => ("diamond", "#adebbc", id.as_str()),
            Resource::Variable(id) => ("diamond", "#adebbc", id.as_str()),
            Resource::Function(id) => ("diamond", "#adebbc", id.as_str()),

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

        ResourceVisualization {
            id: ["n", &index.to_string()].concat(),
            label: label.to_string(),
            shape: shape.to_string(),
            fill_color: fill_color.to_string(),
        }
    }

    pub fn to_cyto(&self) -> String {
        format!(
            r#"{{ "data": {{ "id": "{id}", "label": "{label}"}} }}"#,
            id = self.id,
            label = self.label.replace('\"', "\\\"")
        )
    }

    pub fn to_dot(&self) -> String {
        format!(
            r#"{id} [shape="{shape}" fillcolor="{fill_color}" label="{label}"]"#,
            id = self.id,
            shape = self.shape,
            fill_color = self.fill_color,
            label = self.label.replace('\"', "\\\"")
        )
    }
}

#[derive(
    Debug, Display, Clone, PartialEq, Eq, Hash, EnumString, JsonSchema, Serialize, Deserialize,
)]
#[serde(rename_all = "camelCase")]
pub enum Relation {
    Assigns,
    Imports,
    Links,
    Embeds,
    Uses,
    Reads,
    Writes,
}

pub struct RelationVisualization {
    from: String,
    to: String,
    label: String,
}

impl RelationVisualization {
    pub fn new(from: usize, to: usize, relation: &Relation) -> RelationVisualization {
        RelationVisualization {
            from: ["n", &from.to_string()].concat(),
            to: ["n", &to.to_string()].concat(),
            label: relation.to_string(),
        }
    }

    pub fn to_cyto(&self) -> String {
        format!(
            r#"{{ "data": {{ "source": "{from}", "target": "{to}", "label": "{label}" }} }}"#,
            from = self.from,
            to = self.to,
            label = self.label.replace('\"', "\\\"")
        )
    }

    pub fn to_dot(&self) -> String {
        format!(
            r#"{from} -> {to} [label="{label}"]"#,
            from = self.from,
            to = self.to,
            label = self.label.replace('\"', "\\\"")
        )
    }
}

/// The direction to represent the relation from subject to object.
pub enum Direction {
    From,
    To,
}

pub fn direction(relation: &Relation) -> Direction {
    match relation {
        Relation::Assigns => Direction::To,
        Relation::Imports => Direction::From,
        Relation::Links => Direction::To,
        Relation::Embeds => Direction::From,
        Relation::Uses => Direction::From,
        Relation::Reads => Direction::From,
        Relation::Writes => Direction::To,
    }
}

pub type Triple = (Resource, Relation, Resource);

#[derive(Debug, Default, Clone)]
pub struct Graph {
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
    pub fn to_viz(&self) -> (Vec<ResourceVisualization>, Vec<RelationVisualization>) {
        let nodes = self
            .indices
            .iter()
            .map(|(resource, node)| ResourceVisualization::new(node.index(), resource))
            .collect();

        let edges = self
            .graph
            .edge_indices()
            .filter_map(|edge| {
                if let (Some((from, to)), Some(relation)) = (
                    self.graph.edge_endpoints(edge),
                    self.graph.edge_weight(edge),
                ) {
                    Some(RelationVisualization::new(
                        from.index(),
                        to.index(),
                        relation,
                    ))
                } else {
                    None
                }
            })
            .collect();

        (nodes, edges)
    }

    /// Convert the graph to a Cytoscape graph
    ///
    /// Currently, this outputs a standalone HTML page but the intension is to have an updating,
    /// interactive, navigable view of the graph (by sending node & edge additions/removals to it)
    pub fn to_cyto(&self) -> String {
        let (nodes, edges) = self.to_viz();
        let nodes = nodes
            .iter()
            .map(|node| node.to_cyto())
            .collect::<Vec<String>>()
            .join(", ");
        let edges = edges
            .iter()
            .map(|edge| edge.to_cyto())
            .collect::<Vec<String>>()
            .join(", ");

        format!(
            r#"
<html>
    <head>
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, user-scalable=no, initial-scale=1, maximum-scale=1">
        <script src="https://unpkg.com/cytoscape/dist/cytoscape.min.js"></script>
    </head>
    <body>
        <div id="cy"></div>
        <style>
            #cy {{
                width: 100%;
                height: 100%;
                display: block;
            }}
        </style>
        <script>
            var cy = cytoscape({{
                container: document.getElementById('cy'),
                style: [
                    {{
                        selector: 'node',
                        style: {{
                            'label': 'data(label)',
                            'shape': 'rectangle',
                            'background-color': '#dddddd'
                        }}
                    }},
                    {{
                        selector: 'edge',
                        style: {{
                            'label': 'data(label)',
                            'target-arrow-shape': 'triangle'
                        }}
                    }}
                ],
                layout: {{
                    name: 'breadthfirst',
                    directed: true,
                    padding: 10
                }},
                elements: {{
                    nodes: [{nodes}],
                    edges: [{edges}]
                }}
            }});
        </script>
    </body>
</html>
"#,
            nodes = nodes,
            edges = edges
        )
    }

    /// Convert the graph to a DOT visualization language `digraph`.
    pub fn to_dot(&self) -> String {
        let (nodes, edges) = self.to_viz();
        let nodes = nodes
            .iter()
            .map(|node| node.to_dot())
            .collect::<Vec<String>>()
            .join("\n  ");
        let edges = edges
            .iter()
            .map(|edge| edge.to_dot())
            .collect::<Vec<String>>()
            .join("\n  ");

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
