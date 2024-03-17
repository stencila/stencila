use codec_losses::lost_options;

use crate::{prelude::*, ReplaceBlock};

impl MarkdownCodec for ReplaceBlock {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id))
            .push_str("~~");

        if let Some(status) = &self.suggestion_status {
            context
                .push_str(" ")
                .push_prop_str("suggestion_status", &status.to_string().to_lowercase());
        }

        context
            .push_str("\n\n")
            .push_prop_fn("content", |context| self.content.to_markdown(context))
            .push_str("~>\n\n")
            .push_prop_fn("replacement", |context| {
                self.replacement.to_markdown(context)
            })
            .push_str("~~\n")
            .exit_node()
            .newline();
    }
}
