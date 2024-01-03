use crate::{Block, Inline, Paragraph, Section};

impl From<Vec<Block>> for Block {
    fn from(blocks: Vec<Block>) -> Block {
        Block::Section(Section {
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
