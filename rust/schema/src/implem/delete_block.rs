use codec_losses::lost_options;

use crate::{prelude::*, DeleteBlock, SuggestionStatus};

impl MarkdownCodec for DeleteBlock {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id))
            .push_semis()
            .push_str(" delete");

        if let Some(status @ (SuggestionStatus::Accepted | SuggestionStatus::Rejected)) =
            &self.suggestion_status
        {
            context
                .push_str(" ")
                .push_prop_str("suggestion_status", &status.to_string().to_lowercase());
        }

        context
            .push_str("\n\n")
            .push_prop_fn("content", |context| self.content.to_markdown(context))
            .push_semis()
            .newline()
            .exit_node()
            .newline();
    }
}
