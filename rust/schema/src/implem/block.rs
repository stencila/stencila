use crate::{Block, Inline, Paragraph, StyledBlock};

impl From<Vec<Block>> for Block {
    fn from(blocks: Vec<Block>) -> Block {
        Block::StyledBlock(StyledBlock {
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
