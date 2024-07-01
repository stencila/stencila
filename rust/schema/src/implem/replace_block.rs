use codec_info::lost_options;

use crate::{prelude::*, ReplaceBlock};

impl MarkdownCodec for ReplaceBlock {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id))
            .push_semis()
            .push_str(" replace");

        if let Some(feedback) = &self.feedback {
            context
                .push_str(" ")
                .push_prop_str(NodeProperty::Feedback, feedback);
        }

        if self.content.is_empty() {
            context.newline();
        } else {
            context
                .push_str("\n\n")
                .push_prop_fn(NodeProperty::Content, |context| {
                    self.content.to_markdown(context)
                });
        }

        context
            .push_semis()
            .push_str(" with\n\n")
            .increase_depth()
            .push_prop_fn(NodeProperty::Replacement, |context| {
                self.replacement.to_markdown(context)
            })
            .decrease_depth()
            .push_semis()
            .newline()
            .exit_node()
            .newline();
    }
}
