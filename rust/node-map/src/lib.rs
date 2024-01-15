use std::collections::HashMap;

use common::{
    derive_more::{Deref, DerefMut},
    serde::Serialize,
    smol_str::SmolStr,
};
use schema::{
    walk::{Visitor, WalkControl, WalkNode},
    Block, Inline, Node, NodeId,
};

/// Walk over a node to generate a mapping of `NodeId`s to paths within the node
pub fn node_map<T: WalkNode>(node: &T) -> HashMap<NodeId, NodePath> {
    let mut mapper = Mapper::default();
    mapper.visit(node);
    mapper.map
}

/// The path to a node within another node
#[derive(Default, Clone, Serialize, Deref, DerefMut)]
#[serde(crate = "common::serde")]
pub struct NodePath(Vec<NodePathSegment>);

/// A segment in a node path
#[derive(Clone, Serialize)]
#[serde(untagged, crate = "common::serde")]
pub enum NodePathSegment {
    Property(SmolStr),
    Index(usize),
}

/// A visitor that collects node ids and addresses
#[derive(Default)]
struct Mapper {
    /// The current path in the root node
    path: NodePath,

    /// The collected mapping between node ids and paths
    map: HashMap<NodeId, NodePath>,
}

impl Visitor for Mapper {
    /// Visit a `Node` node type
    fn visit_node(&mut self, node: &Node) -> WalkControl {
        if let Some(node_id) = node.node_id() {
            self.map.insert(node_id, self.path.clone());
        }
        WalkControl::Continue
    }

    /// Visit a `Block` node type
    fn visit_block(&mut self, block: &Block) -> WalkControl {
        if let Some(node_id) = block.node_id() {
            self.map.insert(node_id, self.path.clone());
        }
        WalkControl::Continue
    }

    /// Visit an `Inline` node type
    fn visit_inline(&mut self, inline: &Inline) -> WalkControl {
        if let Some(node_id) = inline.node_id() {
            self.map.insert(node_id, self.path.clone());
        }
        WalkControl::Continue
    }

    /// Enter a property
    fn enter_property(&mut self, name: &str) {
        self.path.push(NodePathSegment::Property(name.into()));
    }

    /// Exit a property
    fn exit_property(&mut self) {
        self.path.pop();
    }

    /// Enter a node at an index
    fn enter_index(&mut self, index: usize) {
        self.path.push(NodePathSegment::Index(index));
    }

    /// Exit a node at an index
    fn exit_index(&mut self) {
        self.path.pop();
    }
}
