use crate::{prelude::*, ChatMessageGroup};

impl ChatMessageGroup {
    /// Custom implementation of [`PatchNode::apply`] to ensure only one message
    /// in the group is selected
    pub fn apply_patch_op(
        &mut self,
        path: &mut PatchPath,
        op: &PatchOp,
        _context: &mut PatchContext,
    ) -> Result<bool> {
        if let (
            Some(PatchSlot::Property(NodeProperty::Messages)),
            Some(PatchSlot::Index(which)),
            Some(PatchSlot::Property(NodeProperty::IsSelected)),
            PatchOp::Set(value),
        ) = (path.front(), path.get(1), path.get(2), op)
        {
            if let Ok(true) = bool::from_value(value.clone()) {
                for (index, message) in self.messages.iter_mut().enumerate() {
                    message.is_selected = (index == *which).then_some(true);
                }
                return Ok(true);
            }
        }

        Ok(false)
    }
}

impl MarkdownCodec for ChatMessageGroup {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .push_colons()
            .push_str(" messages")
            .newline()
            .newline()
            .push_prop_fn(NodeProperty::Messages, |context| {
                self.messages.to_markdown(context)
            })
            .push_colons()
            .newline()
            .exit_node()
            .newline();
    }
}
