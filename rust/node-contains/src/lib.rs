use stencila_schema::{
    Block, Citation, IfBlockClause, Inline, ListItem, Node, NodeId, NodeProperty, NodeType,
    SuggestionBlock, SuggestionInline, TableCell, TableRow, Visitor, WalkControl, WalkNode,
    WalkthroughStep,
};

/// Determine whether a node contains any of a set of node ids
///
/// A more performant alternative to using multiple calls to `node_find::find`
/// because (1) it breaks the walk when the first node id is found, (2) it
/// does not clone the found node.
///
/// Returns the first node id found.
pub fn contains<T>(node: &T, node_ids: Vec<NodeId>) -> Option<NodeId>
where
    T: WalkNode,
{
    let mut finder = Walker {
        node_ids,
        node_id: None,
    };
    finder.walk(node);
    finder.node_id
}

/// A visitor that walks over a node and returns the first of the node ids found
struct Walker {
    node_ids: Vec<NodeId>,
    node_id: Option<NodeId>,
}

impl Walker {
    /// Break walk if node has been found
    fn walk_control(&self) -> WalkControl {
        match self.node_id {
            Some(..) => WalkControl::Break,
            None => WalkControl::Continue,
        }
    }
}

impl Visitor for Walker {
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
        if let Some(node_id) = node.node_id()
            && self.node_ids.contains(&node_id)
        {
            self.node_id = Some(node_id);
            return WalkControl::Break;
        }

        WalkControl::Continue
    }

    fn visit_block(&mut self, block: &Block) -> WalkControl {
        if let Some(node_id) = block.node_id()
            && self.node_ids.contains(&node_id)
        {
            self.node_id = Some(node_id);
            return WalkControl::Break;
        }

        WalkControl::Continue
    }

    fn visit_inline(&mut self, inline: &Inline) -> WalkControl {
        if let Some(node_id) = inline.node_id()
            && self.node_ids.contains(&node_id)
        {
            self.node_id = Some(node_id);
            return WalkControl::Break;
        }

        WalkControl::Continue
    }

    fn visit_citation(&mut self, citation: &Citation) -> WalkControl {
        let node_id = citation.node_id();
        if self.node_ids.contains(&node_id) {
            self.node_id = Some(node_id);
            return WalkControl::Break;
        }

        WalkControl::Continue
    }

    fn visit_if_block_clause(&mut self, clause: &IfBlockClause) -> WalkControl {
        let node_id = clause.node_id();
        if self.node_ids.contains(&node_id) {
            self.node_id = Some(node_id);
            return WalkControl::Break;
        }

        WalkControl::Continue
    }

    fn visit_list_item(&mut self, list_item: &ListItem) -> WalkControl {
        let node_id = list_item.node_id();
        if self.node_ids.contains(&node_id) {
            self.node_id = Some(node_id);
            return WalkControl::Break;
        }

        WalkControl::Continue
    }

    fn visit_suggestion_block(&mut self, block: &SuggestionBlock) -> WalkControl {
        let node_id = block.node_id();
        if self.node_ids.contains(&node_id) {
            self.node_id = Some(node_id);
            return WalkControl::Break;
        }

        WalkControl::Continue
    }

    fn visit_suggestion_inline(&mut self, inline: &SuggestionInline) -> WalkControl {
        let node_id = inline.node_id();
        if self.node_ids.contains(&node_id) {
            self.node_id = Some(node_id);
            return WalkControl::Break;
        }

        WalkControl::Continue
    }

    fn visit_table_row(&mut self, table_row: &TableRow) -> WalkControl {
        let node_id = table_row.node_id();
        if self.node_ids.contains(&node_id) {
            self.node_id = Some(node_id);
            return WalkControl::Break;
        }

        WalkControl::Continue
    }

    fn visit_table_cell(&mut self, table_cell: &TableCell) -> WalkControl {
        let node_id = table_cell.node_id();
        if self.node_ids.contains(&node_id) {
            self.node_id = Some(node_id);
            return WalkControl::Break;
        }

        WalkControl::Continue
    }

    fn visit_walkthrough_step(&mut self, walkthrough_step: &WalkthroughStep) -> WalkControl {
        let node_id = walkthrough_step.node_id();
        if self.node_ids.contains(&node_id) {
            self.node_id = Some(node_id);
            return WalkControl::Break;
        }

        WalkControl::Continue
    }
}
