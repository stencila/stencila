use codec_losses::lost_options;

use crate::{prelude::*, ListItem};

impl MarkdownCodec for ListItem {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id, item, position))
            .push_prop_fn("content", |context| self.content.to_markdown(context));

        context.exit_node();

        /*
        let checkbox = self.is_checked.map(|is_checked| match is_checked {
            true => t("[x] "),
            false => t("[ ] "),
        });

        let (md, mut losses) = match checkbox {
            Some(checkbox) => {
                // Check box is only added is the first block is a paragraph
                if let Some(Block::Paragraph(paragraph)) = self.content.first() {
                    let mut paragraph = paragraph.clone();
                    paragraph.content.insert(0, checkbox);

                    let (mut md, mut losses) = paragraph.to_markdown(context);
                    let (rest_md, rest_losses) = self.content[1..].to_vec().to_markdown(context);

                    md.push_str(&rest_md);
                    losses.merge(rest_losses);

                    (md, losses)
                } else {
                    self.content.to_markdown(context)
                }
            }
            None => self.content.to_markdown(context),
        };
        */
    }
}
