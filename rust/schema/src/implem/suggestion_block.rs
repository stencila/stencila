use stencila_codec_info::lost_options;

use crate::{SuggestionBlock, SuggestionType, implem::suggestion::suggestion_attrs, prelude::*};

impl SuggestionBlock {
    pub fn to_jats_special(&self) -> (String, Losses) {
        let (content, mut losses) = self.content.to_jats();

        losses.add("SuggestionBlock@");

        (content, losses)
    }
}

impl MarkdownCodec for SuggestionBlock {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        if matches!(
            context.mode,
            MarkdownEncodeMode::Clean | MarkdownEncodeMode::Render
        ) || !matches!(context.format, Format::Smd)
        {
            context.enter_node(self.node_type(), self.node_id());

            match self.suggestion_type {
                Some(SuggestionType::Delete) => {
                    context.push_prop_fn(NodeProperty::Content, |context| {
                        self.content.to_markdown(context)
                    });
                }
                Some(SuggestionType::Replace) => {
                    context.push_prop_fn(NodeProperty::Original, |context| {
                        self.original.to_markdown(context)
                    });
                }
                _ => {}
            }

            context.exit_node();
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

        if let Some(attrs) = suggestion_attrs(&self.authors, &self.date_published) {
            context.push_str(&attrs);
        }

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

        match self.suggestion_type {
            Some(SuggestionType::Replace) => {
                context
                    .push_prop_fn(NodeProperty::Original, |context| {
                        self.original.to_markdown(context)
                    })
                    .push_str("\n\n:~>\n\n")
                    .push_prop_fn(NodeProperty::Content, |context| {
                        self.content.to_markdown(context)
                    });
            }
            _ => {
                context.push_prop_fn(NodeProperty::Content, |context| {
                    self.content.to_markdown(context)
                });
            }
        }

        context.push_str(fence).newline();

        context.exit_node().newline();
    }
}
