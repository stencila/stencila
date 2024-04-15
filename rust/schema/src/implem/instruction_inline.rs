use codec_losses::{lost_exec_options, lost_options};

use crate::{prelude::*, InstructionInline};

impl MarkdownCodec for InstructionInline {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id, auto_exec))
            .merge_losses(lost_exec_options!(self))
            .push_str("[[do ");

        if let Some(assignee) = &self.assignee {
            context.push_str("@").push_str(assignee).push_str(" ");
        }

        if let Some(part) = self
            .messages
            .first()
            .and_then(|message| message.parts.first())
        {
            context.push_prop_fn("message", |context| part.to_markdown(context));
        }

        if let Some(content) = &self.content {
            context
                .push_str(">>")
                .push_prop_fn("content", |context| content.to_markdown(context));
        };

        context.push_str("]]");

        if let Some(suggestion) = &self.options.suggestion {
            context.push_prop_fn("suggestion", |context| suggestion.to_markdown(context));
        }

        context.exit_node();
    }
}
