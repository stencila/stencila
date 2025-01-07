use codec::{
    common::eyre::{bail, Result},
    schema::{Article, Node},
    DecodeInfo, EncodeInfo,
};

use crate::{
    blocks::{blocks_from_lexical, blocks_to_lexical}, lexical::{LexicalDoc, RootNode}, shared::{LexicalDecodeContext, LexicalEncodeContext}
};

/// Decode a Lexical document to a Stencila root node
pub(crate) fn root_from_lexical(lexical: LexicalDoc) -> Result<(Node, DecodeInfo)> {
    let mut context = LexicalDecodeContext::default();
    let root = node_from_lexical(lexical, &mut context)?;

    Ok((
        root,
        DecodeInfo {
            losses: context.losses,
            ..Default::default()
        },
    ))
}

/// Encode a Stencila root node to a Lexical document
pub(crate) fn root_to_lexical(root: &Node) -> Result<(LexicalDoc, EncodeInfo)> {
    let mut context = LexicalEncodeContext::default();
    let lexical = node_to_lexical(root, &mut context)?;

    Ok((
        lexical,
        EncodeInfo {
            losses: context.losses,
            ..Default::default()
        },
    ))
}

fn node_from_lexical(lexical: LexicalDoc, context: &mut LexicalDecodeContext) -> Result<Node> {
    let article = article_from_lexical(lexical, context);
    Ok(Node::Article(article))
}

fn node_to_lexical(node: &Node, context: &mut LexicalEncodeContext) -> Result<LexicalDoc> {
    match node {
        Node::Article(article) => Ok(article_to_lexical(article, context)),
        _ => bail!("Unsupported node type: {}", node.node_type()),
    }
}

fn article_from_lexical(lexical: LexicalDoc, context: &mut LexicalDecodeContext) -> Article {
    Article {
        content: blocks_from_lexical(lexical.root.children, context),
        ..Default::default()
    }
}

fn article_to_lexical(article: &Article, context: &mut LexicalEncodeContext) -> LexicalDoc {
    LexicalDoc {
        root: RootNode {
            children: blocks_to_lexical(&article.content, context),
            ..Default::default()
        },
    }
}
