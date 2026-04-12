use stencila_codec_info::lost_options;

use crate::{SuggestionInline, SuggestionType, prelude::*};

impl MarkdownCodec for SuggestionInline {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        if matches!(context.mode, MarkdownEncodeMode::Clean) {
            context
                .enter_node(self.node_type(), self.node_id())
                .push_prop_fn(NodeProperty::Content, |context| {
                    if self.suggestion_type == Some(SuggestionType::Delete) {
                        self.content.to_markdown(context);
                    }
                })
                .exit_node();
            return;
        }

        // If rendering, or format is anything other than Stencila Markdown, skip encoding
        // and record as loss
        if matches!(context.mode, MarkdownEncodeMode::Render)
            || !matches!(context.format, Format::Smd)
        {
            context.losses.add(self.node_type().to_string());

            return;
        }

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
