use stencila_codec_info::lost_options;

use crate::{SuggestionInline, SuggestionType, prelude::*};

impl MarkdownCodec for SuggestionInline {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        let is_delete = self.suggestion_type == Some(SuggestionType::Delete);
        let (open, close) = if is_delete {
            ("{--", "--}")
        } else {
            ("{++", "++}")
        };

        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id))
            .push_str(open)
            .push_prop_fn(NodeProperty::Content, |context| {
                self.content.to_markdown(context)
            })
            .push_str(close)
            .exit_node();
    }
}
