use crate::{transforms::blocks_to_inlines, Block, Call, Division, Include, Inline, Paragraph};

impl From<Vec<Block>> for Block {
    fn from(blocks: Vec<Block>) -> Block {
        Block::Division(Division {
            content: blocks,
            ..Default::default()
        })
    }
}

impl From<Vec<Inline>> for Block {
    fn from(inlines: Vec<Inline>) -> Block {
        Block::Paragraph(Paragraph {
            content: inlines,
            ..Default::default()
        })
    }
}

impl Into<Vec<Inline>> for Block {
    fn into(self) -> Vec<Inline> {
        match &self {
            // Variants with inline content
            Block::Heading(heading) => heading.content.to_owned(),
            Block::Paragraph(paragraph) => paragraph.content.to_owned(),

            // Variants with block content
            Block::Claim(claim) => blocks_to_inlines(claim.content.to_owned()),
            Block::Include(Include { content, .. }) | Block::Call(Call { content, .. }) => {
                match &content {
                    Some(content) => blocks_to_inlines(content.to_owned()),
                    None => vec![self.into()],
                }
            }

            // Fallback to a single item vector of `self` transformed to an inline
            _ => vec![self.into()],
        }
    }
}
