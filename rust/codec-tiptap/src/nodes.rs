//! Root-node conversion between Tiptap documents and Stencila nodes.

use stencila_codec::{
    DecodeInfo, EncodeInfo,
    eyre::{Result, bail},
    stencila_schema::{Article, Node},
};

use crate::{
    blocks::{blocks_from_tiptap, blocks_to_tiptap},
    shared::{TiptapDecodeContext, TiptapEncodeContext},
    tiptap::TiptapDoc,
};

/// Decode a Tiptap document to a Stencila root node.
pub(crate) fn root_from_tiptap(tiptap: TiptapDoc) -> Result<(Node, DecodeInfo)> {
    let mut context = TiptapDecodeContext::default();
    let root = Node::Article(Article {
        content: blocks_from_tiptap(tiptap.content, &mut context),
        ..Default::default()
    });

    Ok((
        root,
        DecodeInfo {
            losses: context.losses,
            ..Default::default()
        },
    ))
}

/// Encode a Stencila root node to a Tiptap document.
pub(crate) fn root_to_tiptap(root: &Node) -> Result<(TiptapDoc, EncodeInfo)> {
    let mut context = TiptapEncodeContext::default();

    let Node::Article(article) = root else {
        bail!("Unsupported node type: {}", root.node_type());
    };

    let doc = TiptapDoc {
        content: blocks_to_tiptap(&article.content, &mut context),
        ..Default::default()
    };

    Ok((
        doc,
        EncodeInfo {
            losses: context.losses,
            ..Default::default()
        },
    ))
}
