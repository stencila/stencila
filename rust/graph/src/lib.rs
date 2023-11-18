//! Types for representing the inter-dependencies between nodes in a document

use std::collections::HashMap;

use petgraph::{
    stable_graph::{NodeIndex, StableGraph},
    visit::{EdgeRef, IntoEdgeReferences, IntoNodeReferences},
};

use common::{
    derivative::Derivative,
    eyre::{bail, eyre, Report, Result},
    serde::{Serialize, Serializer},
    serde_json::{self, json},
    serde_yaml,
};
use format::Format;
use schema::{
    Button, Call, CodeChunk, CodeExpression, Division, ExecutionDependantNode,
    ExecutionDependantRelation, ExecutionDependencyNode, ExecutionDependencyRelation, File,
    Function, Parameter, SoftwareSourceCode, Span, Variable,
};

/// The nodes in the graph
///
/// Represents the union of the variants in `ExecutionDependencyNode`
/// and `ExecutionDependantNode`. Rather than use the nodes themselves
/// (which are large, and not `Eq` and `Hash` which is necessary for `HashMap` keys)
/// this `enum`` represents each of the relevant `Node` types using key
/// identifying properties.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
#[serde(crate = "common::serde")]
pub enum GraphNode {
    Button {
        id: String,
    },
    Call {
        id: String,
    },
    CodeChunk {
        id: String,
    },
    CodeExpression {
        id: String,
    },
    Division {
        id: String,
    },
    File {
        path: String,
    },
    Function {
        name: String,
    },
    Parameter {
        name: String,
    },
    Span {
        id: String,
    },
    SoftwareSourceCode {
        name: String,
        programming_language: String,
    },
    Variable {
        name: String,
    },
}

impl TryFrom<&ExecutionDependencyNode> for GraphNode {
    type Error = Report;

    fn try_from(value: &ExecutionDependencyNode) -> Result<Self> {
        type Other = ExecutionDependencyNode;
        Ok(match value {
            Other::Button(Button { id, .. }) => Self::Button {
                id: id.clone().ok_or_else(|| eyre!("Button missing id"))?,
            },
            Other::CodeChunk(CodeChunk { id, .. }) => Self::CodeChunk {
                id: id.clone().ok_or_else(|| eyre!("CodeChunk missing id"))?,
            },
            Other::File(File { path, .. }) => Self::File { path: path.clone() },
            Other::Parameter(Parameter { name, .. }) => Self::Parameter { name: name.clone() },
            Other::SoftwareSourceCode(SoftwareSourceCode {
                name,
                programming_language,
                ..
            }) => Self::SoftwareSourceCode {
                name: name.clone(),
                programming_language: programming_language.clone(),
            },
            Other::Variable(Variable { name, .. }) => Self::Variable { name: name.clone() },
        })
    }
}

impl TryFrom<&ExecutionDependantNode> for GraphNode {
    type Error = Report;

    fn try_from(value: &ExecutionDependantNode) -> Result<Self> {
        type Other = ExecutionDependantNode;
        Ok(match value {
            Other::Button(Button { id, .. }) => Self::Button {
                id: id.clone().ok_or_else(|| eyre!("Button missing id"))?,
            },
            Other::Call(Call { id, .. }) => Self::Call {
                id: id.clone().ok_or_else(|| eyre!("Call missing id"))?,
            },
            Other::CodeChunk(CodeChunk { id, .. }) => Self::CodeChunk {
                id: id.clone().ok_or_else(|| eyre!("CodeChunk missing id"))?,
            },
            Other::CodeExpression(CodeExpression { id, .. }) => Self::CodeExpression {
                id: id
                    .clone()
                    .ok_or_else(|| eyre!("CodeExpression missing id"))?,
            },
            Other::Division(Division { id, .. }) => Self::Division {
                id: id.clone().ok_or_else(|| eyre!("Division missing id"))?,
            },
            Other::File(File { path, .. }) => Self::File { path: path.clone() },
            Other::Function(Function { name, .. }) => Self::Function { name: name.clone() },
            Other::Parameter(Parameter { name, .. }) => Self::Parameter { name: name.clone() },
            Other::Span(Span { id, .. }) => Self::Span {
                id: id.clone().ok_or_else(|| eyre!("Span missing id"))?,
            },
            Other::Variable(Variable { name, .. }) => Self::Variable { name: name.clone() },
        })
    }
}

/// The edges between nodes in the graph
///
/// Represents the union of the variants in `ExecutionDependencyRelation`
/// and `ExecutionDependantRelation`.
#[derive(Debug, Serialize)]
#[serde(crate = "common::serde")]
pub enum GraphEdge {
    Alters,
    Assigns,
    Calls,
    Declares,
    Derives,
    Imports,
    Includes,
    Reads,
    Uses,
    Writes,
}

impl From<&ExecutionDependencyRelation> for GraphEdge {
    fn from(value: &ExecutionDependencyRelation) -> Self {
        match value {
            ExecutionDependencyRelation::Calls => Self::Calls,
            ExecutionDependencyRelation::Derives => Self::Derives,
            ExecutionDependencyRelation::Imports => Self::Imports,
            ExecutionDependencyRelation::Includes => Self::Includes,
            ExecutionDependencyRelation::Reads => Self::Reads,
            ExecutionDependencyRelation::Uses => Self::Uses,
        }
    }
}

impl From<&ExecutionDependantRelation> for GraphEdge {
    fn from(value: &ExecutionDependantRelation) -> Self {
        match value {
            ExecutionDependantRelation::Assigns => Self::Assigns,
            ExecutionDependantRelation::Alters => Self::Alters,
            ExecutionDependantRelation::Declares => Self::Declares,
            ExecutionDependantRelation::Writes => Self::Writes,
        }
    }
}

#[derive(Derivative, Default)]
#[derivative(Debug)]
pub struct Graph {
    /// The indices of the nodes in the graph
    ///
    /// It is necessary to store [`NodeIndex`] for each `[Node]`
    /// so we can keep track of which nodes are already in the
    /// graph and re-use their index if they are.
    indices: HashMap<GraphNode, NodeIndex>,

    /// The graph itself
    ///
    /// Use a `petgraph::StableGraph` so that nodes can be added and removed
    /// without changing node indices.
    #[derivative(Debug = "ignore")]
    graph: StableGraph<GraphNode, GraphEdge>,
}

impl Graph {
    /// Add a dependency relation between two nodes
    pub fn add_dependency(
        &mut self,
        to: &ExecutionDependantNode,
        relation: &ExecutionDependencyRelation,
        from: &ExecutionDependencyNode,
    ) -> Result<()> {
        let from = GraphNode::try_from(from)?;
        let from = if let Some(index) = self.indices.get(&from) {
            *index
        } else {
            let index = self.graph.add_node(from.clone());
            self.indices.insert(from, index);
            index
        };

        let to = GraphNode::try_from(to)?;
        let to = if let Some(index) = self.indices.get(&to) {
            *index
        } else {
            let index = self.graph.add_node(to.clone());
            self.indices.insert(to, index);
            index
        };

        let edge = GraphEdge::from(relation);

        self.graph.add_edge(from, to, edge);

        Ok(())
    }

    /// Add a dependant relation between two nodes
    pub fn add_dependant(
        &mut self,
        from: &ExecutionDependencyNode,
        relation: &ExecutionDependantRelation,
        to: &ExecutionDependantNode,
    ) -> Result<()> {
        let from = GraphNode::try_from(from)?;
        let from = if let Some(index) = self.indices.get(&from) {
            *index
        } else {
            let index = self.graph.add_node(from.clone());
            self.indices.insert(from, index);
            index
        };

        let to = GraphNode::try_from(to)?;
        let to = if let Some(index) = self.indices.get(&to) {
            *index
        } else {
            let index = self.graph.add_node(to.clone());
            self.indices.insert(to, index);
            index
        };

        let edge = GraphEdge::from(relation);

        self.graph.add_edge(from, to, edge);

        Ok(())
    }

    /// Convert the graph to some format
    pub fn to_format(&self, format: Format) -> Result<String> {
        use Format::*;
        Ok(match format {
            Json => serde_json::to_string_pretty(self)?,
            Yaml => serde_yaml::to_string(self)?,
            // TODO: Add dot and d2
            _ => bail!("Graphs can not be serialized to format '{}'", format),
        })
    }
}

impl Serialize for Graph {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let nodes: Vec<serde_json::Value> = self
            .graph
            .node_references()
            .map(|(index, node)| {
                let mut obj = serde_json::to_value(node).expect("Should serialize");
                let obj = obj.as_object_mut().expect("Should be an object");
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

        json!({
            "nodes": nodes,
            "edges": edges
        })
        .serialize(serializer)
    }
}
