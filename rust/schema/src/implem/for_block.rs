use codec_losses::lost_exec_options;

use crate::{prelude::*, ForBlock};

impl MarkdownCodec for ForBlock {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_exec_options!(self));

        let fence = ":".repeat(3 + context.depth * 2);

        context
            .push_str(&fence)
            .push_str(" for ")
            .push_prop_str("symbol", &self.symbol)
            .push_str(" in ")
            .push_prop_str("code", &self.code);

        if let Some(lang) = &self.programming_language {
            if !lang.is_empty() {
                context
                    .push_str("{")
                    .push_prop_str("programming_language", lang)
                    .push_str("}");
            }
        }

        context
            .push_str("\n\n")
            .increase_depth()
            .push_prop_fn("content", |context| self.content.to_markdown(context))
            .decrease_depth();

        if let Some(otherwise) = &self.otherwise {
            context
                .push_str(&fence)
                .push_str(" else\n\n")
                .increase_depth()
                .push_prop_fn("otherwise", |context| otherwise.to_markdown(context))
                .decrease_depth();
        }

        context
            .push_str(&fence)
            .push_str("\n")
            .exit_node()
            .push_str("\n");
    }
}
