use codec_info::lost_options;

use crate::{prelude::*, Admonition};

impl MarkdownCodec for Admonition {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id));

        context
            .push_str("> [!")
            .push_prop_str(
                NodeProperty::AdmonitionType,
                &self.admonition_type.to_string().to_lowercase(),
            )
            .push_str("]");

        if let Some(is_folded) = self.is_folded {
            context.push_prop_str(NodeProperty::IsFolded, if is_folded { "+" } else { "-" });
        }

        if let Some(title) = &self.title {
            context
                .push_str(" ")
                .push_prop_fn(NodeProperty::Title, |context| title.to_markdown(context));
        }

        context
            .newline()
            .push_line_prefix("> ")
            .prefix_empty_lines(true)
            .push_prop_fn(NodeProperty::Content, |context| {
                self.content.to_markdown(context)
            })
            .pop_line_prefix();

        if context.content.ends_with("> \n") {
            context.content.pop();
            context.content.pop();
            context.content.pop();
            context.content.push('\n');
        }

        context.exit_node();
    }
}
