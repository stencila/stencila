use codec_info::lost_options;

use crate::{prelude::*, Block, ListItem};

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
