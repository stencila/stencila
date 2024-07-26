use codec_info::lost_options;

use crate::{prelude::*, SuggestionInline, SuggestionStatus};

impl MarkdownCodec for SuggestionInline {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        // Do not encode accepted or rejected suggestions
        if matches!(
            self.suggestion_status,
            Some(SuggestionStatus::Accepted | SuggestionStatus::Rejected),
        ) {
            return;
        }

        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id))
            .push_str("[[suggest ")
            .push_prop_fn(NodeProperty::Content, |context| {
                self.content.to_markdown(context)
            })
            .push_str("]]")
            .exit_node();
    }
}
