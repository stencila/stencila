use codec_info::{lost_exec_options, lost_options};

use crate::{prelude::*, InstructionBlock};

impl MarkdownCodec for InstructionBlock {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        if context.render {
            // Encode content only
            if let Some(content) = &self.content {
                content.to_markdown(context);
            }
            return;
        }

        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id, auto_exec))
            .merge_losses(lost_exec_options!(self))
            .push_semis()
            .push_str(" do ");

        if let Some(assignee) = &self.assignee {
            context.push_str("@").push_str(assignee).push_str(" ");
        }

        if let Some(part) = self
            .messages
            .last()
            .and_then(|message| message.parts.first())
        {
            context
                .push_prop_fn(NodeProperty::Messages, |context| part.to_markdown(context))
                .newline();
        }

        if let Some(content) = &self.content {
            context
                .push_semis()
                .push_str(" with\n\n")
                .push_prop_fn(NodeProperty::Content, |context| {
                    content.to_markdown(context)
                })
                .push_semis()
                .newline();
        };

        context.newline();

        if let Some(suggestion) = &self.suggestion {
            context.push_prop_fn(NodeProperty::Suggestion, |context| {
                suggestion.to_markdown(context)
            });
        }

        context.exit_node();
    }
}
