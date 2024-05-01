//! Handling of text document synchronization related messages
//!
//! See https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_synchronization

use std::{ops::ControlFlow, sync::Arc};

use async_lsp::{
    lsp_types::{
        DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
        DidSaveTextDocumentParams, Range,
    },
    Error,
};

use codecs::{DecodeOptions, EncodeInfo, EncodeOptions, Format};
use common::{
    tokio::{
        self,
        sync::{mpsc, RwLock},
    },
    tracing,
};
use schema::{NodeId, NodeType, Visitor};

use crate::{inspect::Inspector, ServerState};

/// A Stencila `Node` within a `TextDocument`
///
/// This mirrors the structure of a document but only recording the attributes needed for
/// deriving code lenses, document symbols etc.
#[derive(Debug, Clone)]
pub(super) struct TextNode {
    /// The range in the document that the node occurs
    pub range: Range,

    /// The type of the node
    pub node_type: NodeType,

    /// The id of the node
    pub node_id: NodeId,

    /// A string detail of the node
    pub detail: Option<String>,

    /// The children of the node
    pub children: Vec<TextNode>,
}

impl Default for TextNode {
    fn default() -> Self {
        Self {
            range: Range::default(),
            node_type: NodeType::Article,
            node_id: NodeId::null(),
            detail: None,
            children: Vec::new(),
        }
    }
}

/// An iterator over text nodes
pub(super) struct TextNodeIterator<'a> {
    items: Vec<&'a TextNode>,
}

impl<'a> TextNodeIterator<'a> {
    pub fn new(root: &'a TextNode) -> Self {
        TextNodeIterator { items: vec![root] }
    }
}

impl<'a> Iterator for TextNodeIterator<'a> {
    type Item = &'a TextNode;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(node) = self.items.pop() {
            // Push children to the stack in reverse order to ensure they are processed in the correct order
            for child in node.children.iter().rev() {
                self.items.push(child);
            }
            Some(node)
        } else {
            None
        }
    }
}

impl TextNode {
    /// Get the node and it descendants as a list
    pub fn flatten(&self) -> TextNodeIterator {
        TextNodeIterator::new(self)
    }
}

/// A text document that has been opened by the language server
pub(super) struct TextDocument {
    /// A sender to the `update_task`
    ///
    /// This is an `UnboundedSender` to that updates can be sent from
    ///
    sender: mpsc::UnboundedSender<String>,

    /// The nodes in the text document
    ///
    /// This is updated in the `update_task`
    pub root: Arc<RwLock<TextNode>>,
}

impl TextDocument {
    /// Create a new text document with an initial source
    fn new(source: String) -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        let root = Arc::new(RwLock::new(TextNode::default()));

        let root_clone = root.clone();
        tokio::spawn(async {
            Self::update_task(receiver, root_clone).await;
        });

        if let Err(error) = sender.send(source) {
            tracing::error!("While sending initial source: {error}");
        }

        Self { sender, root }
    }

    /// Update the text document with new text content
    fn update(&self, source: String) {
        if let Err(error) = self.sender.send(source) {
            tracing::error!("While sending updated source: {error}");
        }
    }

    /// The async background task which updates the document and inspects it to
    /// enumerate nodes, diagnostics etc
    async fn update_task(
        mut receiver: mpsc::UnboundedReceiver<String>,
        root: Arc<RwLock<TextNode>>,
    ) {
        while let Some(source) = receiver.recv().await {
            // Take a write lock on the root node so that readers can not read
            // until the update is finished
            let mut root = root.write().await;

            // Decode the document
            let node = codecs::from_str(
                &source,
                Some(DecodeOptions {
                    format: Some(Format::Markdown),
                    ..Default::default()
                }),
            )
            .await
            .unwrap(); // TODO: record diagnostic if this fails

            // Encode the document to get generated content and mapping
            let (generated, EncodeInfo { mapping, .. }) = codecs::to_string_with_info(
                &node,
                Some(EncodeOptions {
                    format: Some(Format::Markdown),
                    ..Default::default()
                }),
            )
            .await
            .unwrap(); // TODO: record diagnostic if this fails

            // Walk the node to enumerate nodes and diagnostics within it
            let mut inspector = Inspector::new(&source, &generated, mapping);
            inspector.visit(&node);

            // Update the document's nodes etc
            if let Some(node) = inspector.root() {
                *root = node;
            }
        }
    }
}

/// Handle a notification from the client that a text document was opened
pub(super) fn did_open(
    state: &mut ServerState,
    params: DidOpenTextDocumentParams,
) -> ControlFlow<Result<(), Error>> {
    state.documents.insert(
        params.text_document.uri.to_string(),
        TextDocument::new(params.text_document.text),
    );

    ControlFlow::Continue(())
}

/// Handle a notification from the client that a text document was changes
pub(super) fn did_change(
    state: &mut ServerState,
    params: DidChangeTextDocumentParams,
) -> ControlFlow<Result<(), Error>> {
    let uri = params.text_document.uri.to_string();
    if let Some(doc) = state.documents.get_mut(&uri) {
        // TODO: This assumes a whole document change (with TextDocumentSyncKind::FULL in initialize):
        // needs more defensiveness and potentially implement incremental sync
        doc.update(params.content_changes[0].text.clone());
    } else {
        tracing::warn!("Unknown document `${uri}`")
    }

    ControlFlow::Continue(())
}

/// Handle a notification from the client that a text document was save
pub(super) fn did_save(
    _state: &mut ServerState,
    _params: DidSaveTextDocumentParams,
) -> ControlFlow<Result<(), Error>> {
    ControlFlow::Continue(())
}

/// Handle a notification from the client that a text document was closed
pub(super) fn did_close(
    state: &mut ServerState,
    params: DidCloseTextDocumentParams,
) -> ControlFlow<Result<(), Error>> {
    state
        .documents
        .remove(&params.text_document.uri.to_string());

    ControlFlow::Continue(())
}
