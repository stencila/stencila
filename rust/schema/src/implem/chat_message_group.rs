use crate::{prelude::*, ChatMessageGroup};

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
