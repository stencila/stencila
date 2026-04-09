use stencila_codec_info::lost_options;

use crate::{SuggestionBlock, SuggestionType, prelude::*};

impl SuggestionBlock {
    pub fn to_jats_special(&self) -> (String, Losses) {
        let (content, mut losses) = self.content.to_jats();

        losses.add("SuggestionBlock@");

        (content, losses)
    }
}

impl MarkdownCodec for SuggestionBlock {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id));

        // If rendering, or format is anything other than Stencila Markdown or
        // MyST, then encode `content` only (if any)
        if context.render || !matches!(context.format, Format::Smd) {
            context
                .push_prop_fn(NodeProperty::Content, |context| {
                    self.content.to_markdown(context)
                })
                .exit_node();

            return;
        }

        let fence = if self.suggestion_type == Some(SuggestionType::Delete) {
            ":--"
        } else {
            ":++"
        };

        context.push_str(fence);

        if let Some(status) = &self.suggestion_status {
            context
                .push_str(" ")
                .push_prop_str(NodeProperty::SuggestionStatus, status.to_keyword());
        }

        if let Some(feedback) = &self.feedback {
            context
                .push_str(" ")
                .push_prop_str(NodeProperty::Feedback, feedback);
        }

        context
            .push_str("\n\n")
            .push_prop_fn(NodeProperty::Content, |context| {
                self.content.to_markdown(context)
            })
            .push_str(fence)
            .newline();

        context.exit_node().newline();
    }
}
