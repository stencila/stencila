use stencila_schema::{BlockContent, InlineContent, Paragraph};

pub trait ToVecBlockContent {
    fn to_vec_block_content(&self) -> Vec<BlockContent>;
}

impl ToVecBlockContent for Vec<InlineContent> {
    /// Coerce a vector of `InlineContent` nodes into a vector of `BlockContent` nodes
    /// by simply wrapping them in a `Paragraph`
    fn to_vec_block_content(&self) -> Vec<BlockContent> {
        vec![BlockContent::Paragraph(Paragraph {
            content: self.clone(),
            ..Default::default()
        })]
    }
}
