use std::collections::HashMap;

use kernel_kuzu::kuzu::{LogicalType, Value};
use schema::{
    Article, Block, IfBlockClause, Inline, ListItem, Node, NodeId, NodePath, NodeProperty,
    NodeSlot, NodeType, TableCell, TableRow, Visitor, WalkControl,
};

use super::{DatabaseNode, NodeAncestors};

/// A visitor which collects entries for node and relation tables from a node
///
/// Walks over a [`Node`] and collects the results of [`DatabaseNode::node_table`] and
/// [`DatabaseNode::rel_tables`]. These results are normalized into the `node_tables` and
/// `rel_tables` hash maps which are optimized for having one prepared statement for
/// each entry.
#[derive(Default)]
#[allow(clippy::type_complexity)]
pub struct DatabaseWalker {
    /// The position (relative index of block of inline node) in the walk
    position: usize,

    /// The current path in the walk
    node_path: NodePath,

    /// The current ancestors in the walk
    node_ancestors: NodeAncestors,

    /// Entries for the database node tables
    pub node_tables: HashMap<
        NodeType,
        (
            Vec<(NodeProperty, LogicalType)>,
            Vec<(usize, NodePath, NodeAncestors, NodeId, Vec<Value>)>,
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
                self.position,
                self.node_path.clone(),
                self.node_ancestors.clone(),
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

    /// Visit several [`DatabaseNode`]s
    ///
    /// Note that this does not walk each node it just visits each.
    pub(crate) fn visit_database_nodes<T>(&mut self, nodes: &[T])
    where
        T: DatabaseNode,
    {
        for node in nodes {
            self.visit_database_node(node);
        }
    }
}

impl Visitor for DatabaseWalker {
    fn enter_struct(&mut self, node_type: NodeType, _node_id: NodeId) -> WalkControl {
        if !matches!(node_type, NodeType::Section) {
            self.node_ancestors.push(node_type.to_string());
        }
        WalkControl::Continue
    }

    fn exit_struct(&mut self) {
        self.node_ancestors.pop();
    }

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
        // Visit nodes that are not otherwise walked over
        if let Node::Article(Article {
            authors: Some(authors),
            references,
            ..
        }) = node
        {
            for author in authors {
                self.visit_database_node(author);
            }

            for reference in references.iter().flatten() {
                for author in reference.authors.iter().flatten() {
                    self.visit_database_node(author);
                }
                self.visit_database_node(reference);
            }
        }

        self.visit_database_node(node)
    }

    fn visit_block(&mut self, node: &Block) -> WalkControl {
        self.position += 1;

        // Special handling for section as ancestor. See `enter_struct` method
        // for the default.
        if let Block::Section(section) = node {
            let ancestor = section
                .section_type
                .as_ref()
                .map(|section_type| section_type.to_string())
                .unwrap_or_else(|| "Section".to_string());
            self.node_ancestors.push(ancestor);
        }

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
        self.visit_database_node(node);
        // Break the walk so that the content (usually just a single Paragraph)
        // is not collected. We do this to reduce the number of nodes in the db.
        WalkControl::Break
    }

    fn visit_inline(&mut self, node: &Inline) -> WalkControl {
        self.position += 1;

        self.visit_database_node(node)
    }
}
