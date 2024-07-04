use codec_info::lost_options;

use crate::{prelude::*, SuggestionBlock};

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
            context.push_semis().push_str(" suggest");

            if let Some(feedback) = &self.feedback {
                context
                    .push_str(" ")
                    .push_prop_str(NodeProperty::Feedback, feedback);
            }

            context
                .push_str("\n\n")
                .increase_depth()
                .push_prop_fn(NodeProperty::Content, |context| {
                    self.content.to_markdown(context)
                })
                .decrease_depth()
                .push_semis()
                .newline();
        }
        context.exit_node().newline();
    }
}
