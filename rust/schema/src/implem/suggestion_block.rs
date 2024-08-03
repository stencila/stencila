use codec_info::lost_options;

use crate::{prelude::*, SuggestionBlock, SuggestionStatus};

impl SuggestionBlock {
    pub fn to_jats_special(&self) -> (String, Losses) {
        let (content, mut losses) = self.content.to_jats();

        losses.add("SuggestionBlock@");

        (content, losses)
    }
}

impl MarkdownCodec for SuggestionBlock {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        // Only encode proposed suggestions to Markdown
        if !matches!(self.suggestion_status, SuggestionStatus::Proposed) {
            return;
        }

        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id));

        if matches!(context.format, Format::Myst) {
            context.myst_directive(
                ':',
                "suggest",
                |context| {
                    if let Some(feedback) = &self.feedback {
                        context
                            .push_str(" ")
                            .push_prop_str(NodeProperty::Feedback, feedback);
                    }
                },
                |_| {},
                |context| {
                    context.push_prop_fn(NodeProperty::Content, |context| {
                        self.content.to_markdown(context)
                    });
                },
            );
        } else {
            context.push_colons().push_str(" suggest");

            if let Some(feedback) = &self.feedback {
                context
                    .push_str(" ")
                    .push_prop_str(NodeProperty::Feedback, feedback);
            }

            if self.content.is_empty() {
                context.push_str(" <");
            } else {
                if self.content.len() == 1 {
                    context.push_str(" >");
                }

                context
                    .push_str("\n\n")
                    .increase_depth()
                    .push_prop_fn(NodeProperty::Content, |context| {
                        self.content.to_markdown(context)
                    })
                    .decrease_depth();

                if self.content.len() > 1 {
                    context.push_colons().newline();
                }
            }
        }

        context.exit_node().newline();
    }
}
