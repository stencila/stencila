use std::collections::HashMap;

use pandoc_types::definition::Pandoc;

use codec::{
    common::eyre::{bail, Result},
    schema::*,
    DecodeInfo, EncodeInfo,
};

use crate::{
    blocks::{blocks_from_pandoc, blocks_to_pandoc},
    shared::{PandocDecodeContext, PandocEncodeContext},
};

pub fn root_to_pandoc(root: &Node) -> Result<(Pandoc, EncodeInfo)> {
    let mut context = PandocEncodeContext::default();
    let pandoc = node_to_pandoc(root, &mut context)?;

    Ok((
        pandoc,
        EncodeInfo {
            losses: context.losses,
            ..Default::default()
        },
    ))
}

pub fn root_from_pandoc(pandoc: Pandoc) -> Result<(Node, DecodeInfo)> {
    let mut context = PandocDecodeContext::default();
    let node = node_from_pandoc(pandoc, &mut context)?;

    Ok((
        node,
        DecodeInfo {
            losses: context.losses,
            ..Default::default()
        },
    ))
}

fn node_to_pandoc(node: &Node, context: &mut PandocEncodeContext) -> Result<Pandoc> {
    match node {
        Node::Article(article) => Ok(article_to_pandoc(article, context)),
        _ => bail!("Unsupported node type: {}", node.node_type()),
    }
}

fn node_from_pandoc(pandoc: Pandoc, context: &mut PandocDecodeContext) -> Result<Node> {
    let article = article_from_pandoc(pandoc, context);
    Ok(Node::Article(article))
}

fn article_to_pandoc(article: &Article, context: &mut PandocEncodeContext) -> Pandoc {
    // TODO: construct Pandoc metadata from article
    let meta = HashMap::new();
    let blocks = blocks_to_pandoc(&article.content, context);

    Pandoc { meta, blocks }
}

fn article_from_pandoc(pandoc: Pandoc, context: &mut PandocDecodeContext) -> Article {
    // TODO: extract article properties from article metadata
    let content = blocks_from_pandoc(pandoc.blocks, context);

    Article {
        content,
        ..Default::default()
    }
}
