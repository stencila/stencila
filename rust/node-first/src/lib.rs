use stencila_schema::{
    Block, Citation, IfBlockClause, Inline, ListItem, Node, NodeId, NodeProperty, NodeType,
    SuggestionBlock, SuggestionInline, TableCell, TableRow, Visitor, WalkControl, WalkNode,
    WalkthroughStep,
};

/// Get the first node of one or more [`NodeType`]s within another node
pub fn first<T>(node: &T, node_types: &[NodeType]) -> Option<Node>
where
    T: WalkNode,
{
    let mut finder = Walker {
        node_types,
        node: None,
    };
    finder.walk(node);
    finder.node
}

/// A visitor that walks over a node and attempts to find a descendant
/// that is one of the node types
struct Walker<'lt> {
    node_types: &'lt [NodeType],
    node: Option<Node>,
}

impl Walker<'_> {
    /// Break walk if node has been found
    fn walk_control(&self) -> WalkControl {
        match self.node {
            Some(..) => WalkControl::Break,
            None => WalkControl::Continue,
        }
    }
}

impl Visitor for Walker<'_> {
    fn enter_struct(&mut self, _node_type: NodeType, _node_id: NodeId) -> WalkControl {
        self.walk_control()
    }

    fn enter_property(&mut self, _property: NodeProperty) -> WalkControl {
        self.walk_control()
    }

    fn enter_index(&mut self, _index: usize) -> WalkControl {
        self.walk_control()
    }

    fn visit_node(&mut self, node: &Node) -> WalkControl {
        if self.node_types.contains(&node.node_type()) {
            self.node = Some(node.clone());
            return WalkControl::Break;
        }

        WalkControl::Continue
    }

    fn visit_block(&mut self, block: &Block) -> WalkControl {
        if self.node_types.contains(&block.node_type()) {
            self.node = Some(block.clone().into());
            return WalkControl::Break;
        }

        WalkControl::Continue
    }

    fn visit_inline(&mut self, inline: &Inline) -> WalkControl {
        if self.node_types.contains(&inline.node_type()) {
            self.node = Some(inline.clone().into());
            return WalkControl::Break;
        }

        WalkControl::Continue
    }

    fn visit_citation(&mut self, citation: &Citation) -> WalkControl {
        if self.node_types.contains(&citation.node_type()) {
            self.node = Some(Node::Citation(citation.clone()));
            return WalkControl::Break;
        }

        WalkControl::Continue
    }

    fn visit_if_block_clause(&mut self, clause: &IfBlockClause) -> WalkControl {
        if self.node_types.contains(&clause.node_type()) {
            self.node = Some(Node::IfBlockClause(clause.clone()));
            return WalkControl::Break;
        }

        WalkControl::Continue
    }

    fn visit_list_item(&mut self, list_item: &ListItem) -> WalkControl {
        if self.node_types.contains(&list_item.node_type()) {
            self.node = Some(Node::ListItem(list_item.clone()));
            return WalkControl::Break;
        }

        WalkControl::Continue
    }

    fn visit_suggestion_block(&mut self, block: &SuggestionBlock) -> WalkControl {
        if self.node_types.contains(&block.node_type()) {
            self.node = Some(Node::SuggestionBlock(block.clone()));
            return WalkControl::Break;
        }

        WalkControl::Continue
    }

    fn visit_suggestion_inline(&mut self, inline: &SuggestionInline) -> WalkControl {
        if self.node_types.contains(&inline.node_type()) {
            self.node = Some(Node::SuggestionInline(inline.clone()));
            return WalkControl::Break;
        }

        WalkControl::Continue
    }

    fn visit_table_row(&mut self, table_row: &TableRow) -> WalkControl {
        if self.node_types.contains(&table_row.node_type()) {
            self.node = Some(Node::TableRow(table_row.clone()));
            return WalkControl::Break;
        }

        WalkControl::Continue
    }

    fn visit_table_cell(&mut self, table_cell: &TableCell) -> WalkControl {
        if self.node_types.contains(&table_cell.node_type()) {
            self.node = Some(Node::TableCell(table_cell.clone()));
            return WalkControl::Break;
        }

        WalkControl::Continue
    }

    fn visit_walkthrough_step(&mut self, walkthrough_step: &WalkthroughStep) -> WalkControl {
        if self.node_types.contains(&walkthrough_step.node_type()) {
            self.node = Some(Node::WalkthroughStep(walkthrough_step.clone()));
            return WalkControl::Break;
        }

        WalkControl::Continue
    }
}
