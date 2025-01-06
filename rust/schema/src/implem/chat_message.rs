use crate::{prelude::*, ChatMessage, MessageRole};

impl MarkdownCodec for ChatMessage {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .push_colons()
            .push_prop_str(
                NodeProperty::Role,
                match self.role {
                    MessageRole::User => " chat/user",
                    MessageRole::System => " chat/system",
                    MessageRole::Model => " chat/model",
                },
            )
            .newline()
            .newline()
            .increase_depth()
            .push_prop_fn(NodeProperty::Content, |context| {
                self.content.to_markdown(context)
            })
            .decrease_depth()
            .push_colons()
            .newline()
            .exit_node()
            .newline();
    }
}
