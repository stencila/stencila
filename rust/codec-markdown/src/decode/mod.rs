use codec::{
    common::eyre::{bail, Result},
    schema::{Article, Node},
    DecodeOptions, Losses,
};

mod blocks;
mod content;
mod frontmatter;
mod inlines;
mod parse;

pub use content::{decode_blocks, decode_inlines};
use frontmatter::decode_frontmatter;

/// Decode a Markdown string to a Stencila Schema [`Node`]
pub(super) fn decode(md: &str, _options: Option<DecodeOptions>) -> Result<(Node, Losses)> {
    let (end, node) = decode_frontmatter(md)?;

    let md = &md[end..];

    let mut node = match node {
        Some(node) => node,
        None => Node::Article(Article::default()),
    };

    let (content, losses) = decode_blocks(md);
    if !content.is_empty() {
        match &mut node {
            Node::Article(article) => article.content = content,
            _ => bail!("Unsupported node type {:?}", node),
        }
    }

    Ok((node, losses))
}
