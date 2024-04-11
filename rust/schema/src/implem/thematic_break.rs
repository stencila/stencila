use crate::{prelude::*, ThematicBreak};

impl PatchNode for ThematicBreak {
    fn condense(&self, context: &mut CondenseContext) {
        // Because a ThematicBreak does not have any properties,
        // unless we add this pseudo-source property it does not
        // get included in the condense context.
        context
            .enter_node(self.node_type(), self.node_id())
            .enter_property(NodeProperty::Type)
            .collect_value("")
            .exit_property()
            .exit_node();
    }
}
