//! Publishing of diagnostics and other notifications
//!
//! See https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_publishDiagnostics

use async_lsp::{
    lsp_types::{notification::Notification, Range, Url},
    ClientSocket,
};

use common::{
    serde::{Deserialize, Serialize},
    tracing,
};
use schema::{ExecutionStatus, NodeType};

use crate::text_document::TextNode;

/// Information about a node for generating gutter decorations
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
struct NodeInfo {
    /// The range of the node
    range: Range,

    /// The type of the node
    node_type: String,

    /// The execution status of the node
    execution_status: Option<ExecutionStatus>,
}

struct PublishNodeInfo;

#[derive(Serialize, Deserialize)]
#[serde(crate = "common::serde")]
struct PublishNodeInfoParams {
    uri: Url,
    nodes: Vec<NodeInfo>,
}

impl Notification for PublishNodeInfo {
    const METHOD: &'static str = "stencila/publishNodeInfo";
    type Params = PublishNodeInfoParams;
}

/// Publish node information
pub(super) fn publish(uri: &Url, text_node: &TextNode, client: &mut ClientSocket) {
    let mut nodes = Vec::new();
    node_infos(text_node, &mut nodes);

    if let Err(error) = client.notify::<PublishNodeInfo>(PublishNodeInfoParams {
        uri: uri.clone(),
        nodes,
    }) {
        tracing::error!("While publishing node info notifications: {error}");
    }
}

/// Create [`NodeInfo`] for a [`TextNode`]
fn node_infos(node: &TextNode, items: &mut Vec<NodeInfo>) {
    // Do not send info for nodes without a range (i.e. that are not encoded to the text document)
    // or for inlines (or their children)
    if (node.range == Range::default()
        || (!node.is_block
            && !matches!(
                node.node_type,
                NodeType::IfBlockClause | NodeType::Walkthrough | NodeType::WalkthroughStep
            )))
        && !matches!(node.node_type, NodeType::Article | NodeType::Prompt)
    {
        return;
    }

    // Do not publish node info for root nodes and some other
    // container types which are effectively "look-through" for the user
    if !matches!(
        node.node_type,
        NodeType::Article
            | NodeType::Prompt
            | NodeType::Chat
            | NodeType::IfBlockClause
            | NodeType::SuggestionBlock
            | NodeType::Walkthrough
            | NodeType::WalkthroughStep
    ) {
        items.push(NodeInfo {
            range: node.range,
            node_type: node.node_type.to_string(),
            execution_status: node.execution.as_ref().and_then(|exec| exec.status.clone()),
        });
    }

    for child in &node.children {
        node_infos(child, items);
    }
}
