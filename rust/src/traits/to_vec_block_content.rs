use stencila_schema::{BlockContent, InlineContent, Node, Paragraph};

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

impl ToVecBlockContent for Node {
    fn to_vec_block_content(&self) -> Vec<BlockContent> {
        match self {
            Node::Article(node) => match &node.content {
                Some(content) => content.clone(),
                None => Vec::new(),
            },
            _ => Vec::new(),
        }
    }
}
