use codec_info::lost_options;

use crate::{prelude::*, ReplaceBlock, SuggestionStatus};

impl MarkdownCodec for ReplaceBlock {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id))
            .push_semis()
            .push_str(" replace");

        if let Some(status @ (SuggestionStatus::Accepted | SuggestionStatus::Rejected)) =
            &self.suggestion_status
        {
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

        context
            .push_semis()
            .push_str(" with\n\n")
            .push_prop_fn("replacement", |context| {
                self.replacement.to_markdown(context)
            })
            .push_semis()
            .newline()
            .exit_node()
            .newline();
    }
}
