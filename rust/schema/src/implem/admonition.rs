use codec_losses::lost_options;

use crate::{prelude::*, Admonition};

impl MarkdownCodec for Admonition {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id));

        context
            .push_str("> [!")
            .push_prop_str(
                "admonition_type",
                &self.admonition_type.to_string().to_lowercase(),
            )
            .push_str("]");

        if let Some(is_folded) = self.is_folded {
            context.push_prop_str("is_folded", if is_folded { "+" } else { "-" });
        }

        if let Some(title) = &self.title {
            context
                .push_str(" ")
                .push_prop_fn("title", |context| title.to_markdown(context));
        }

        context.push_str("\n").increase_depth();
        let line_prefix = "> ".repeat(context.depth);

        context
            .push_line_prefix(&line_prefix)
            .push_prop_fn("content", |context| self.content.to_markdown(context))
            .pop_line_prefix()
            .decrease_depth()
            .push_str("\n")
            .exit_node()
            .push_str("\n");
    }
}
