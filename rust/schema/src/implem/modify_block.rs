use codec_losses::lost_options;

use crate::{prelude::*, ModifyBlock, ModifyOperation};

impl MarkdownCodec for ModifyBlock {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id))
            .push_semis()
            .push_str(" modify");

        if let Some(status) = &self.suggestion_status {
            context
                .push_str(" ")
                .push_prop_str("suggestion_status", &status.to_string().to_lowercase());
        }

        if self.content.is_empty() {
            context.newline();
        } else {
            context
                .push_str("\n\n")
                .push_prop_fn("content", |context| self.content.to_markdown(context));
        }

        context.push_semis().push_str(" with\n\n");

        let modified =
            ModifyOperation::apply_many(&self.operations, &self.content).unwrap_or_default();
        context
            .push_prop_fn("operations", |context| modified.to_markdown(context))
            .push_semis()
            .newline()
            .exit_node()
            .newline();
    }
}
