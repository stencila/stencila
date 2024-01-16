use codec_losses::{lost_exec_options, lost_options};

use crate::{prelude::*, InstructionBlock};

impl MarkdownCodec for InstructionBlock {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id, auto_exec))
            .merge_losses(lost_exec_options!(self))
            .push_str("%% ");

        if let Some(assignee) = &self.options.assignee {
            context.push_str("@").push_str(assignee).push_str(" ");
        }

        if let Some(part) = self
            .messages
            .first()
            .and_then(|message| message.parts.first())
        {
            context
                .push_prop_fn("message", |context| part.to_markdown(context))
                .push_str("\n");
        }

        if let Some(content) = &self.content {
            context
                .push_str("%>\n\n")
                .push_prop_fn("content", |context| content.to_markdown(context))
                .push_str("%%\n");
        };

        context.push_str("\n");

        if let Some(suggestion) = &self.options.suggestion {
            context.push_prop_fn("suggestion", |context| suggestion.to_markdown(context));
        }

        context.exit_node();
    }
}
