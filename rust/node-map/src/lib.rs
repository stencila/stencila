use common::{
    derive_more::{Deref, DerefMut},
    indexmap::IndexMap,
    serde::Serialize,
    smol_str::{SmolStr, ToSmolStr},
};
use schema::{
    Block, Inline, ListItem, Node, NodeId, NodeProperty, TableCell, TableRow, Visitor, WalkControl, WalkNode
};

/// Walk over a node to generate a mapping of `NodeId`s to paths within the node
pub fn node_map<T: WalkNode>(node: &T) -> IndexMap<NodeId, NodePath> {
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
    map: IndexMap<NodeId, NodePath>,
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

    /// Visit a `ListItem` node
    fn visit_list_item(&mut self, list_item: &ListItem) -> WalkControl {
        self.map.insert(list_item.node_id(), self.path.clone());
        WalkControl::Continue
    }

    /// Visit a `TableRow` node
    fn visit_table_row(&mut self, table_row: &TableRow) -> WalkControl {
        self.map.insert(table_row.node_id(), self.path.clone());
        WalkControl::Continue
    }

    /// Visit a `TableCell` node
    fn visit_table_cell(&mut self, table_cell: &TableCell) -> WalkControl {
        self.map.insert(table_cell.node_id(), self.path.clone());
        WalkControl::Continue
    }

    /// Enter a property
    fn enter_property(&mut self, property: NodeProperty) {
        self.path.push(NodePathSegment::Property(property.to_smolstr()));
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
