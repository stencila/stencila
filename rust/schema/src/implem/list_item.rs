use codec_info::lost_options;

use crate::{Block, ListItem, Paragraph, prelude::*};

impl LatexCodec for ListItem {
    fn to_latex(&self, context: &mut LatexEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id, item, position, is_checked));

        if let (1, Some(Block::Paragraph(Paragraph { content, .. }))) =
            (self.content.len(), self.content.first())
        {
            context
                .str("  \\item ")
                .property_fn(NodeProperty::Content, |context| {
                    // Encode paragraph content without ensuring blank line before it
                    context.paragraph_begin();
                    content.to_latex(context);
                    context.paragraph_end();
                });
        } else {
            context
                .str("  \\item\n")
                .property_fn(NodeProperty::Content, |context| {
                    self.content.to_latex(context)
                });
        }

        context.exit_node();
    }
}

impl MarkdownCodec for ListItem {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id, item, position));

        // Check box is only added if the first block is a paragraph
        if let Some(is_checked) = self.is_checked {
            if let Some(Block::Paragraph(..)) = self.content.first() {
                context.push_str(if is_checked { "[x] " } else { "[ ] " });
            }
        }

        context
            .push_prop_fn(NodeProperty::Content, |context| {
                self.content.to_markdown(context)
            })
            .exit_node();
    }
}
