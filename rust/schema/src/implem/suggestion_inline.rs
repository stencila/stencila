use codec_info::lost_options;

use crate::{prelude::*, SuggestionInline};

impl MarkdownCodec for SuggestionInline {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
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
