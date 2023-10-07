use crate::{prelude::*, Block, BlocksOrInlines, Inline, ListItem};

impl ListItem {
    pub fn to_markdown_special(&self, context: &MarkdownEncodeContext) -> (String, Losses) {
        let checkbox = self.is_checked.map(|is_checked| match is_checked {
            true => Inline::String("[x] ".to_string()),
            false => Inline::String("[ ] ".to_string()),
        });

        let (md, mut losses) = match &self.content {
            Some(content) => match content {
                BlocksOrInlines::Inlines(inlines) => match checkbox {
                    Some(checkbox) => [vec![checkbox], inlines.clone()]
                        .concat()
                        .to_markdown(context),
                    None => inlines.to_markdown(context),
                },
                BlocksOrInlines::Blocks(blocks) => match checkbox {
                    Some(checkbox) => {
                        // Check box is only added is the first block is a paragraph
                        if let Some(Block::Paragraph(paragraph)) = blocks.first() {
                            let mut paragraph = paragraph.clone();
                            paragraph.content.insert(0, checkbox);

                            let (mut md, mut losses) = paragraph.to_markdown(context);
                            let (rest_md, rest_losses) = blocks[1..].to_vec().to_markdown(context);

                            md.push_str(&rest_md);
                            losses.merge(rest_losses);

                            (md, losses)
                        } else {
                            blocks.to_markdown(context)
                        }
                    }
                    None => blocks.to_markdown(context),
                },
            },
            None => (String::new(), Losses::none()),
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
