use crate::{prelude::*, shortcuts::t, Block, ListItem};

impl ListItem {
    pub fn to_markdown_special(&self, context: &mut MarkdownEncodeContext) -> (String, Losses) {
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

        if self.id.is_some() {
            losses.add("ListItem.id")
        }
        if self.item.is_some() {
            losses.add("ListItem.item")
        }
        if self.position.is_some() {
            losses.add("ListItem.position")
        }

        (md, losses)
    }
}
