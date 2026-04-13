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
        if matches!(context.mode, MarkdownEncodeMode::Clean) {
            context.enter_node(self.node_type(), self.node_id());

            if matches!(
                self.suggestion_type,
                Some(SuggestionType::Delete | SuggestionType::Replace)
            ) {
                context.push_prop_fn(NodeProperty::Content, |context| {
                    if self.suggestion_type == Some(SuggestionType::Replace) {
                        self.original.to_markdown(context)
                    } else {
                        self.content.to_markdown(context)
                    }
                });
            }

            context.exit_node();
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

        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id));

        let fence = match self.suggestion_type {
            Some(SuggestionType::Delete) => ":--",
            Some(SuggestionType::Replace) => ":~~",
            _ => ":++",
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

        context.push_str("\n\n");

        if self.suggestion_type == Some(SuggestionType::Replace) {
            context.push_prop_fn(NodeProperty::Content, |context| {
                self.original.to_markdown(context)
            });
            context.push_str("\n\n:~>\n\n");
        }

        context
            .push_prop_fn(NodeProperty::Content, |context| {
                self.content.to_markdown(context)
            })
            .push_str(fence)
            .newline();

        context.exit_node().newline();
    }
}
