use codec::{
    DecodeInfo, EncodeInfo, EncodeOptions,
    common::eyre::{Result, bail},
    format::Format,
    schema::{Article, Node},
};

use crate::{
    blocks::{blocks_from_lexical, blocks_to_lexical},
    lexical::{self, HtmlNode, LexicalDoc, RootNode},
    shared::{LexicalDecodeContext, LexicalEncodeContext},
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
pub(crate) fn root_to_lexical(
    root: &Node,
    options: &Option<EncodeOptions>,
) -> Result<(LexicalDoc, EncodeInfo)> {
    let format = options
        .as_ref()
        .and_then(|options| options.format.clone())
        .unwrap_or(Format::Lexical);

    let standalone = options
        .as_ref()
        .and_then(|options| options.standalone)
        .unwrap_or(false);

    let mut context = LexicalEncodeContext {
        format,
        standalone,
        ..Default::default()
    };

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
    let children = match context.standalone {
        true => vec![lexical::BlockNode::Html(HtmlNode {
            html: codec_dom::encode(&article.content),
            ..Default::default()
        })],
        false => blocks_to_lexical(&article.content, context),
    };

    LexicalDoc {
        root: RootNode {
            children,
            ..Default::default()
        },
    }
}
