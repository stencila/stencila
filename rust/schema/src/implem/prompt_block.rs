use codec_info::{lost_exec_options, lost_options};

use crate::{prelude::*, PromptBlock};

impl MarkdownCodec for PromptBlock {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        if matches!(context.format, Format::Llmd) {
            // Do not encode at all
            return;
        }

        if context.render {
            // Record any execution messages
            if let Some(messages) = &self.options.execution_messages {
                for message in messages {
                    context.add_message(
                        self.node_type(),
                        self.node_id(),
                        message.level.clone().into(),
                        message.message.to_string(),
                    );
                }
            }

            // Encode content only
            if let Some(content) = &self.content {
                content.to_markdown(context);
            }

            return;
        }

        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id))
            .merge_losses(lost_exec_options!(self));

        if matches!(context.format, Format::Myst) {
            context.myst_directive(
                '`',
                "prompt",
                |context| {
                    context
                        .push_str(" ")
                        .push_prop_str(NodeProperty::Prompt, &self.prompt);
                },
                |_| {},
                |_| {},
            );
        } else {
            context
                .push_colons()
                .push_str(" prompt ")
                .push_prop_str(NodeProperty::Prompt, &self.prompt);
        }

        context.newline().exit_node().newline();
    }
}
