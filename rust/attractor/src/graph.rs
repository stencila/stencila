use std::fmt;

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::error::{AttractorError, AttractorResult};
use crate::types::Duration;

/// A typed attribute value in a graph node or edge.
///
/// Supports the value types defined in §2.4 of the specification:
/// strings, integers, floats, booleans, and durations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AttrValue {
    /// A string value (quoted in DOT).
    String(String),
    /// An integer value.
    Integer(i64),
    /// A floating-point value.
    Float(f64),
    /// A boolean value (`true` or `false`).
    Boolean(bool),
    /// A duration value (e.g., `"900s"`, `"15m"`).
    Duration(Duration),
}

impl AttrValue {
    /// Return the value as a string slice if it is a `String` variant.
    #[must_use]
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }

    /// Return the value as an owned string.
    ///
    /// For `String` variants, returns the contained string.
    /// For other variants, returns the `Display` representation.
    #[must_use]
    pub fn to_string_value(&self) -> String {
        match self {
            Self::String(s) => s.clone(),
            Self::Integer(n) => n.to_string(),
            Self::Float(n) => n.to_string(),
            Self::Boolean(b) => b.to_string(),
            Self::Duration(d) => d.to_string(),
        }
    }

    /// Return the value as an `i64` if it is an `Integer` variant.
    #[must_use]
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Self::Integer(n) => Some(*n),
            _ => None,
        }
    }

    /// Return the value as an `f64` if it is a `Float` variant.
    #[must_use]
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Self::Float(n) => Some(*n),
            _ => None,
        }
    }

    /// Return the value as a `bool` if it is a `Boolean` variant.
    #[must_use]
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    /// Return the value as a `Duration` if it is a `Duration` variant.
    #[must_use]
    pub fn as_duration(&self) -> Option<&Duration> {
        match self {
            Self::Duration(d) => Some(d),
            _ => None,
        }
    }

    /// Return a human-readable type name for this value.
    #[must_use]
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::String(_) => "string",
            Self::Integer(_) => "integer",
            Self::Float(_) => "float",
            Self::Boolean(_) => "boolean",
            Self::Duration(_) => "duration",
        }
    }
}

impl fmt::Display for AttrValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::String(s) => write!(f, "\"{s}\""),
            Self::Integer(n) => write!(f, "{n}"),
            Self::Float(n) => write!(f, "{n}"),
            Self::Boolean(b) => write!(f, "{b}"),
            Self::Duration(d) => write!(f, "{d}"),
        }
    }
}

impl From<&str> for AttrValue {
    fn from(s: &str) -> Self {
        Self::String(s.to_string())
    }
}

impl From<String> for AttrValue {
    fn from(s: String) -> Self {
        Self::String(s)
    }
}

impl From<i64> for AttrValue {
    fn from(n: i64) -> Self {
        Self::Integer(n)
    }
}

impl From<f64> for AttrValue {
    fn from(n: f64) -> Self {
        Self::Float(n)
    }
}

impl From<bool> for AttrValue {
    fn from(b: bool) -> Self {
        Self::Boolean(b)
    }
}

impl From<Duration> for AttrValue {
    fn from(d: Duration) -> Self {
        Self::Duration(d)
    }
}

/// A node in the pipeline graph.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Node {
    /// The unique identifier for this node.
    pub id: String,
    /// Key-value attributes attached to this node.
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub attrs: IndexMap<String, AttrValue>,
}

impl Node {
    /// Create a new node with the given ID and no attributes.
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            attrs: IndexMap::new(),
        }
    }

    /// Get an attribute value by key.
    #[must_use]
    pub fn get_attr(&self, key: &str) -> Option<&AttrValue> {
        self.attrs.get(key)
    }

    /// Get a string attribute value by key.
    ///
    /// Returns `Some` only if the attribute exists and is a `String` variant.
    #[must_use]
    pub fn get_str_attr(&self, key: &str) -> Option<&str> {
        self.attrs.get(key).and_then(AttrValue::as_str)
    }

    /// Get a boolean attribute value by key, defaulting to `false`.
    ///
    /// Returns `true` only if the attribute exists and is `AttrValue::Boolean(true)`.
    #[must_use]
    pub fn get_bool_attr(&self, key: &str) -> bool {
        self.attrs
            .get(key)
            .and_then(AttrValue::as_bool)
            .unwrap_or(false)
    }

    /// Return the node's shape, defaulting to `"box"`.
    #[must_use]
    pub fn shape(&self) -> &str {
        self.get_str_attr("shape").unwrap_or("box")
    }

    /// Return the node's label, falling back to the node ID.
    #[must_use]
    pub fn label(&self) -> &str {
        self.get_str_attr("label").unwrap_or(&self.id)
    }

    /// Return the handler type for this node.
    ///
    /// Resolution order:
    /// 1. Explicit `type` attribute
    /// 2. Start/exit/fail node matched by well-known ID (e.g. `Start`, `End`, `Fail`)
    /// 3. Shape-based mapping per §2.8
    #[must_use]
    pub fn handler_type(&self) -> &str {
        if let Some(explicit) = self.get_str_attr("type") {
            return explicit;
        }
        if Graph::START_IDS.contains(&self.id.as_str()) && self.shape() == "box" {
            return "start";
        }
        if Graph::EXIT_IDS.contains(&self.id.as_str()) && self.shape() == "box" {
            return "exit";
        }
        if Graph::FAIL_IDS.contains(&self.id.as_str()) && self.shape() == "box" {
            return "fail";
        }
        shape_to_handler_type(self.shape())
    }
}

/// Map a DOT shape name to the corresponding handler type per §2.8.
pub(crate) fn shape_to_handler_type(shape: &str) -> &'static str {
    match shape {
        "Mdiamond" => "start",
        "Msquare" => "exit",
        "invtriangle" => "fail",
        "hexagon" | "human" => "wait.human",
        "diamond" => "conditional",
        "component" => "parallel",
        "tripleoctagon" => "parallel.fan_in",
        "parallelogram" => "tool",
        "house" => "stack.manager_loop",
        // "box" and all unknown shapes default to codergen
        _ => "codergen",
    }
}

/// A directed edge in the pipeline graph.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Edge {
    /// The source node ID.
    pub from: String,
    /// The target node ID.
    pub to: String,
    /// Key-value attributes attached to this edge.
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub attrs: IndexMap<String, AttrValue>,
}

impl Edge {
    /// Create a new edge between two nodes with no attributes.
    #[must_use]
    pub fn new(from: impl Into<String>, to: impl Into<String>) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            attrs: IndexMap::new(),
        }
    }

    /// Get an attribute value by key.
    #[must_use]
    pub fn get_attr(&self, key: &str) -> Option<&AttrValue> {
        self.attrs.get(key)
    }

    /// Return the edge label, defaulting to an empty string.
    #[must_use]
    pub fn label(&self) -> &str {
        self.attrs
            .get("label")
            .and_then(AttrValue::as_str)
            .unwrap_or("")
    }

    /// Return the edge condition expression, defaulting to an empty string.
    #[must_use]
    pub fn condition(&self) -> &str {
        self.attrs
            .get("condition")
            .and_then(AttrValue::as_str)
            .unwrap_or("")
    }

    /// Return the edge weight, defaulting to `0`.
    #[must_use]
    pub fn weight(&self) -> i64 {
        self.attrs
            .get("weight")
            .and_then(AttrValue::as_i64)
            .unwrap_or(0)
    }
}

/// A directed graph representing a pipeline definition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Graph {
    /// The name of the graph (from `digraph Name { ... }`).
    pub name: String,
    /// Graph-level attributes (from `graph [...]` or top-level declarations).
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub graph_attrs: IndexMap<String, AttrValue>,
    /// Nodes indexed by their ID, in insertion order.
    pub nodes: IndexMap<String, Node>,
    /// Directed edges in the graph.
    pub edges: Vec<Edge>,
}

impl Graph {
    /// Create a new empty graph with the given name.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            graph_attrs: IndexMap::new(),
            nodes: IndexMap::new(),
            edges: Vec::new(),
        }
    }

    /// Add a node to the graph, keyed by its ID.
    pub fn add_node(&mut self, node: Node) {
        self.nodes.insert(node.id.clone(), node);
    }

    /// Add an edge to the graph.
    pub fn add_edge(&mut self, edge: Edge) {
        self.edges.push(edge);
    }

    /// Get a node by ID.
    #[must_use]
    pub fn get_node(&self, id: &str) -> Option<&Node> {
        self.nodes.get(id)
    }

    /// Get a mutable reference to a node by ID.
    #[must_use]
    pub fn get_node_mut(&mut self, id: &str) -> Option<&mut Node> {
        self.nodes.get_mut(id)
    }

    /// Get a graph-level attribute by key.
    #[must_use]
    pub fn get_graph_attr(&self, key: &str) -> Option<&AttrValue> {
        self.graph_attrs.get(key)
    }

    /// Return all edges originating from the given node.
    #[must_use]
    pub fn outgoing_edges(&self, node_id: &str) -> Vec<&Edge> {
        self.edges.iter().filter(|e| e.from == node_id).collect()
    }

    /// Return all edges targeting the given node.
    #[must_use]
    pub fn incoming_edges(&self, node_id: &str) -> Vec<&Edge> {
        self.edges.iter().filter(|e| e.to == node_id).collect()
    }

    /// Shape identifying a start node.
    pub const START_SHAPE: &'static str = "Mdiamond";

    /// Fallback IDs identifying a start node.
    pub const START_IDS: &'static [&'static str] = &["start", "Start"];

    /// Shape identifying an exit node.
    pub const EXIT_SHAPE: &'static str = "Msquare";

    /// Fallback IDs identifying an exit node.
    pub const EXIT_IDS: &'static [&'static str] = &["exit", "Exit", "end", "End"];

    /// Shape identifying a fail node.
    pub const FAIL_SHAPE: &'static str = "invtriangle";

    /// Fallback IDs identifying a fail node.
    pub const FAIL_IDS: &'static [&'static str] = &["fail", "Fail"];

    /// Check whether a node is a start node (by shape or ID).
    #[must_use]
    pub fn is_start_node(node: &Node) -> bool {
        node.shape() == Self::START_SHAPE || Self::START_IDS.contains(&node.id.as_str())
    }

    /// Check whether a node is an exit node (by shape or ID).
    #[must_use]
    pub fn is_exit_node(node: &Node) -> bool {
        node.shape() == Self::EXIT_SHAPE || Self::EXIT_IDS.contains(&node.id.as_str())
    }

    /// Check whether a node is a fail node (by shape or ID).
    #[must_use]
    pub fn is_fail_node(node: &Node) -> bool {
        node.shape() == Self::FAIL_SHAPE || Self::FAIL_IDS.contains(&node.id.as_str())
    }

    /// Find the start node of the pipeline.
    ///
    /// Looks for a node with shape `Mdiamond` first, then falls back to
    /// a node with ID `start` or `Start`.
    ///
    /// # Errors
    ///
    /// Returns [`AttractorError::NoStartNode`] if no start node is found.
    pub fn find_start_node(&self) -> AttractorResult<&Node> {
        self.find_node_by_shape_or_ids(Self::START_SHAPE, Self::START_IDS)
            .ok_or(AttractorError::NoStartNode)
    }

    /// Find the exit node of the pipeline.
    ///
    /// Looks for a node with shape `Msquare` first, then falls back to
    /// a node with ID `exit`, `Exit`, `end`, or `End`.
    ///
    /// # Errors
    ///
    /// Returns [`AttractorError::NoExitNode`] if no exit node is found.
    pub fn find_exit_node(&self) -> AttractorResult<&Node> {
        self.find_node_by_shape_or_ids(Self::EXIT_SHAPE, Self::EXIT_IDS)
            .ok_or(AttractorError::NoExitNode)
    }

    /// Find a node by shape first, then by candidate IDs as fallback.
    fn find_node_by_shape_or_ids(&self, shape: &str, ids: &[&str]) -> Option<&Node> {
        self.nodes
            .values()
            .find(|n| n.shape() == shape)
            .or_else(|| ids.iter().find_map(|id| self.nodes.get(*id)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handler_type_explicit_type_attr() {
        let mut node = Node::new("n1");
        node.attrs
            .insert("type".into(), AttrValue::String("tool".into()));
        assert_eq!(node.handler_type(), "tool");
    }

    #[test]
    fn handler_type_by_shape() {
        let mut node = Node::new("n1");
        node.attrs
            .insert("shape".into(), AttrValue::String("diamond".into()));
        assert_eq!(node.handler_type(), "conditional");
    }

    #[test]
    fn handler_type_start_by_id() {
        let node = Node::new("Start");
        assert_eq!(node.handler_type(), "start");
    }

    #[test]
    fn handler_type_exit_by_id() {
        for id in ["exit", "Exit", "end", "End"] {
            let node = Node::new(id);
            assert_eq!(node.handler_type(), "exit", "failed for id={id}");
        }
    }

    #[test]
    fn handler_type_fail_by_id() {
        for id in ["fail", "Fail"] {
            let node = Node::new(id);
            assert_eq!(node.handler_type(), "fail", "failed for id={id}");
        }
    }

    #[test]
    fn handler_type_id_match_does_not_override_explicit_shape() {
        let mut node = Node::new("Start");
        node.attrs
            .insert("shape".into(), AttrValue::String("diamond".into()));
        assert_eq!(node.handler_type(), "conditional");
    }

    #[test]
    fn handler_type_box_defaults_to_codergen() {
        let node = Node::new("MyNode");
        assert_eq!(node.handler_type(), "codergen");
    }
}
