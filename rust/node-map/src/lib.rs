use common::indexmap::IndexMap;
use schema::{
    Block, Inline, ListItem, Node, NodeId, NodePath, NodeProperty, NodeSlot, TableCell, TableRow,
    Visitor, WalkControl, WalkNode,
};

/// Generate a mapping of [`NodeId`] to [`NodePath`] within a node
pub fn map<T: WalkNode>(node: &T) -> IndexMap<NodeId, NodePath> {
    let mut mapper = Mapper::default();
    mapper.visit(node);
    mapper.map
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
    fn enter_property(&mut self, property: NodeProperty) -> WalkControl {
        self.path.push_back(NodeSlot::Property(property));
        WalkControl::Continue
    }

    /// Exit a property
    fn exit_property(&mut self) {
        self.path.pop_back();
    }

    /// Enter a node at an index
    fn enter_index(&mut self, index: usize) -> WalkControl {
        self.path.push_back(NodeSlot::Index(index));
        WalkControl::Continue
    }

    /// Exit a node at an index
    fn exit_index(&mut self) {
        self.path.pop_back();
    }
}
