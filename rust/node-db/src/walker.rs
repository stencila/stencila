use std::collections::HashMap;

use kuzu::{LogicalType, Value};

use schema::{
    Block, IfBlockClause, Inline, ListItem, Node, NodeId, NodePath, NodeProperty, NodeSlot,
    NodeType, TableCell, TableRow, Visitor, WalkControl,
};

use super::DatabaseNode;

/// A visitor which collects entries for node and relation tables from a node
///
/// Walks over a [`Node`] and collects the results of [`DatabaseNode::node_table`] and
/// [`DatabaseNode::rel_tables`]. These results are normalized into the `node_tables` and
/// `rel_tables` hash maps which are optimized for having one prepared statement for
/// each entry.
#[derive(Default)]
#[allow(clippy::type_complexity)]
pub struct DatabaseWalker {
    /// The current path in the walk
    node_path: NodePath,

    /// Entries for the database node tables
    pub node_tables: HashMap<
        NodeType,
        (
            Vec<(NodeProperty, LogicalType)>,
            Vec<(NodePath, NodeId, Vec<Value>)>,
        ),
    >,

    /// Entries for the database relation tables
    pub rel_tables: HashMap<(NodeType, NodeProperty, NodeType), Vec<(NodeId, Vec<NodeId>)>>,
}

impl DatabaseWalker {
    /// Visit a [`DatabaseNode`] and insert the results of [`DatabaseNode::node_table`] and
    /// [`DatabaseNode::rel_tables`] into `node_tables` and `rel_tables`.
    fn visit_database_node<T>(&mut self, node: &T) -> WalkControl
    where
        T: DatabaseNode,
    {
        let node_type = node.node_type();
        let node_id = node.node_id();
        let node_table = node.node_table();
        let rel_tables = node.rel_tables();

        self.node_tables
            .entry(node_type)
            .or_insert_with(|| {
                (
                    node_table
                        .iter()
                        .map(|(node_property, logical_type, ..)| {
                            (*node_property, logical_type.clone())
                        })
                        .collect(),
                    Vec::new(),
                )
            })
            .1
            .push((
                self.node_path.clone(),
                node_id.clone(),
                node_table.into_iter().map(|(.., value)| value).collect(),
            ));

        for (node_property, to_nodes) in rel_tables {
            let mut to_node_ids: HashMap<NodeType, Vec<NodeId>> = HashMap::new();
            for (to_node_type, to_node_id) in to_nodes {
                to_node_ids
                    .entry(to_node_type)
                    .or_default()
                    .push(to_node_id);
            }

            for (to_node_type, to_node_ids) in to_node_ids {
                self.rel_tables
                    .entry((node_type, node_property, to_node_type))
                    .or_default()
                    .push((node_id.clone(), to_node_ids));
            }
        }

        WalkControl::Continue
    }
}

impl Visitor for DatabaseWalker {
    fn enter_property(&mut self, property: NodeProperty) -> WalkControl {
        self.node_path.push_back(NodeSlot::Property(property));
        WalkControl::Continue
    }

    fn exit_property(&mut self) {
        self.node_path.pop_back();
    }

    fn enter_index(&mut self, index: usize) -> WalkControl {
        self.node_path.push_back(NodeSlot::Index(index));
        WalkControl::Continue
    }

    fn exit_index(&mut self) {
        self.node_path.pop_back();
    }

    fn visit_node(&mut self, node: &Node) -> WalkControl {
        self.visit_database_node(node)
    }

    fn visit_block(&mut self, node: &Block) -> WalkControl {
        self.visit_database_node(node)
    }

    fn visit_if_block_clause(&mut self, node: &IfBlockClause) -> WalkControl {
        self.visit_database_node(node)
    }

    fn visit_list_item(&mut self, node: &ListItem) -> WalkControl {
        self.visit_database_node(node)
    }

    fn visit_table_row(&mut self, node: &TableRow) -> WalkControl {
        self.visit_database_node(node)
    }

    fn visit_table_cell(&mut self, node: &TableCell) -> WalkControl {
        self.visit_database_node(node)
    }

    fn visit_inline(&mut self, node: &Inline) -> WalkControl {
        self.visit_database_node(node)
    }
}
