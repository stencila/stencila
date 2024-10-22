use schema::{Node, NodeId, Visitor, WalkControl, WalkNode};

/// Find a node with a given [`NodeId`] within another node
pub fn find<T>(node: &T, node_id: NodeId) -> Option<Node>
where
    T: WalkNode,
{
    let mut finder = Finder {
        node_id,
        node: None,
    };
    finder.visit(node);
    finder.node
}

/// A visitor that walks over a node and attempts to find a descendant with the
/// given node id
struct Finder {
    node_id: NodeId,
    node: Option<Node>,
}

impl Visitor for Finder {
    fn visit_node(&mut self, node: &Node) -> WalkControl {
        if let Some(node_id) = node.node_id() {
            if node_id == self.node_id {
                self.node = Some(node.clone());
                return WalkControl::Break;
            }
        }

        WalkControl::Continue
    }

    fn visit_block(&mut self, block: &schema::Block) -> WalkControl {
        if let Some(node_id) = block.node_id() {
            if node_id == self.node_id {
                self.node = Some(block.clone().into());
                return WalkControl::Break;
            }
        }

        WalkControl::Continue
    }

    fn visit_inline(&mut self, inline: &schema::Inline) -> WalkControl {
        if let Some(node_id) = inline.node_id() {
            if node_id == self.node_id {
                self.node = Some(inline.clone().into());
                return WalkControl::Break;
            }
        }

        WalkControl::Continue
    }

    fn visit_if_block_clause(&mut self, clause: &schema::IfBlockClause) -> WalkControl {
        if clause.node_id() == self.node_id {
            self.node = Some(Node::IfBlockClause(clause.clone()));
            return WalkControl::Break;
        }

        WalkControl::Continue
    }

    fn visit_list_item(&mut self, list_item: &schema::ListItem) -> WalkControl {
        if list_item.node_id() == self.node_id {
            self.node = Some(Node::ListItem(list_item.clone()));
            return WalkControl::Break;
        }

        WalkControl::Continue
    }

    fn visit_suggestion_block(&mut self, block: &schema::SuggestionBlock) -> WalkControl {
        if block.node_id() == self.node_id {
            self.node = Some(Node::SuggestionBlock(block.clone()));
            return WalkControl::Break;
        }

        WalkControl::Continue
    }

    fn visit_suggestion_inline(&mut self, inline: &schema::SuggestionInline) -> WalkControl {
        if inline.node_id() == self.node_id {
            self.node = Some(Node::SuggestionInline(inline.clone()));
            return WalkControl::Break;
        }

        WalkControl::Continue
    }

    fn visit_table_row(&mut self, table_row: &schema::TableRow) -> WalkControl {
        if table_row.node_id() == self.node_id {
            self.node = Some(Node::TableRow(table_row.clone()));
            return WalkControl::Break;
        }

        WalkControl::Continue
    }

    fn visit_table_cell(&mut self, table_cell: &schema::TableCell) -> WalkControl {
        if table_cell.node_id() == self.node_id {
            self.node = Some(Node::TableCell(table_cell.clone()));
            return WalkControl::Break;
        }

        WalkControl::Continue
    }
}
