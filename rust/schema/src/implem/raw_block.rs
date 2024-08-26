use codec_info::lost_options;

use crate::{prelude::*, RawBlock};

impl MarkdownCodec for RawBlock {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        if context.render {
            // Encode content if format is Markdown
            if Format::from_name(&self.format) == Format::Markdown {
                context.push_str(&self.content);

                // Add as many newlines to separate from following blocks
                if !self.content.ends_with('\n') {
                    context.newline();
                }
                context.newline();

                return;
            }
        }

        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id));

        context
            .push_str("````")
            .push_prop_str(NodeProperty::Format, &self.format)
            .push_str(" raw\n")
            .push_prop_fn(NodeProperty::Code, |context| {
                self.content.to_markdown(context)
            });

        if !self.content.ends_with('\n') {
            context.newline();
        }

        context.push_str("````\n").exit_node().newline();
    }
}
