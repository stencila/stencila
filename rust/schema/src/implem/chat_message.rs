use crate::{prelude::*, ChatMessage, MessageRole, SoftwareApplication};

impl MarkdownCodec for ChatMessage {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .push_colons()
            .push_prop_str(
                NodeProperty::Role,
                match self.role {
                    MessageRole::User => " msg/user",
                    MessageRole::System => " msg/system",
                    MessageRole::Model => " msg/model",
                },
            );

        if let (true, Some(Author::SoftwareApplication(SoftwareApplication { id: Some(id), .. }))) = (
            matches!(self.role, MessageRole::Model),
            &self.options.author,
        ) {
            // Model id is encoded to Markdown for visibility but
            // is not patched from there
            context.push_str(" [").push_str(id).push_str("]");
        }

        context
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
