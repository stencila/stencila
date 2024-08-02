use codec_info::lost_options;

use crate::{prelude::*, ModifyBlock, ModifyOperation};

impl MarkdownCodec for ModifyBlock {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id))
            .push_colons()
            .push_str(" modify");

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

        context.push_colons().push_str(" with\n\n");

        let modified =
            ModifyOperation::apply_many(&self.operations, &self.content).unwrap_or_default();
        context
            .increase_depth()
            .push_prop_fn(NodeProperty::Operations, |context| {
                modified.to_markdown(context)
            })
            .decrease_depth()
            .push_colons()
            .newline()
            .exit_node()
            .newline();
    }
}
