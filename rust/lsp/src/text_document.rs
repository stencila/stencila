//! Handling of text document synchronization related messages
//!
//! See https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_synchronization

use std::{ops::ControlFlow, sync::Arc};

use async_lsp::{
    lsp_types::{
        DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
        DidSaveTextDocumentParams, Position, Range, Url,
    },
    ClientSocket, Error,
};

use codecs::{DecodeOptions, EncodeInfo, EncodeOptions, Format};
use common::{
    tokio::{
        self,
        sync::{mpsc, watch, RwLock},
    },
    tracing,
};
use document::Document;
use schema::{
    Duration, ExecutionMessage, ExecutionStatus, Node, NodeId, NodeType, ProvenanceCount,
    Timestamp, Visitor,
};

use crate::{diagnostics, inspect::Inspector, ServerState};

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
    ///
    /// Used when creating a document symbol for the node.
    pub detail: Option<String>,

    /// Execution details (for executable nodes only)
    ///
    /// These detail are used to publish diagnostics and status
    /// notifications for the node
    pub execution: Option<TextNodeExecution>,

    /// Provenance details (for nodes with a `provenance` field)
    ///
    /// These detail are used to publish provenance summaries
    /// for the node
    pub provenance: Option<Vec<ProvenanceCount>>,

    /// The children of the node
    pub children: Vec<TextNode>,
}

#[derive(Debug, Clone)]
pub(super) struct TextNodeExecution {
    pub status: ExecutionStatus,
    pub duration: Option<Duration>,
    pub ended: Option<Timestamp>,
    pub messages: Option<Vec<ExecutionMessage>>,
}

impl Default for TextNode {
    fn default() -> Self {
        Self {
            range: Range::default(),
            node_type: NodeType::Article,
            node_id: NodeId::null(),
            detail: None,
            execution: None,
            provenance: None,
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
    /// Get the node a position (if any)
    pub fn node_id_at(&self, position: Position) -> Option<NodeId> {
        if position >= self.range.start && position < self.range.end {
            return Some(self.node_id.clone());
        }

        for child in &self.children {
            if let Some(node_id) = child.node_id_at(position) {
                return Some(node_id);
            }
        }

        None
    }

    /// Get the node and it descendants as a list
    pub fn flatten(&self) -> TextNodeIterator {
        TextNodeIterator::new(self)
    }
}

/// A text document that has been opened by the language server
pub(super) struct TextDocument {
    /// The source text of the document e.g. Markdown
    pub source: Arc<RwLock<String>>,

    /// The root node in the text document
    ///
    /// This is updated in the `update_task`.
    pub root: Arc<RwLock<TextNode>>,

    /// The Stencila document for the text document
    ///
    /// This is also updated in the `update_task`.
    pub doc: Arc<RwLock<Document>>,

    /// A sender to the `update_task`
    ///
    /// Sends new source to the `update_task`. This is an `UnboundedSender`
    /// so that updates can be sent from sync functions
    update_sender: mpsc::UnboundedSender<String>,
}

impl TextDocument {
    /// Create a new text document with an initial source
    fn new(uri: Url, source: String, client: ClientSocket) -> Self {
        let doc = Document::new().unwrap(); // TODO: no unwrap

        let watch_receiver = doc.watch();

        let source_string = source.clone();

        let source = Arc::new(RwLock::new(source));
        let root = Arc::new(RwLock::new(TextNode::default()));
        let doc = Arc::new(RwLock::new(doc));

        let (update_sender, update_receiver) = mpsc::unbounded_channel();
        let source_clone = source.clone();
        let doc_clone = doc.clone();
        tokio::spawn(async {
            Self::update_task(update_receiver, source_clone, doc_clone).await;
        });

        let source_clone = source.clone();
        let root_clone = root.clone();
        tokio::spawn(async move {
            Self::watch_task(watch_receiver, uri, source_clone, root_clone, client).await;
        });

        if let Err(error) = update_sender.send(source_string) {
            tracing::error!("While sending initial source: {error}");
        }

        TextDocument {
            source,
            root,
            doc,
            update_sender,
        }
    }

    /// An async background task which updates the source and
    /// the Stencila document
    async fn update_task(
        mut receiver: mpsc::UnboundedReceiver<String>,
        source: Arc<RwLock<String>>,
        doc: Arc<RwLock<Document>>,
    ) {
        while let Some(new_source) = receiver.recv().await {
            // Update the source
            *source.write().await = new_source.clone();

            // Decode the document
            let node = match codecs::from_str(
                &new_source,
                Some(DecodeOptions {
                    format: Some(Format::Markdown),
                    ..Default::default()
                }),
            )
            .await
            {
                Ok(node) => node,
                Err(error) => {
                    tracing::error!("While decoding document: {error}");
                    continue;
                }
            };

            // Update the Stencila document with the new node
            let doc = doc.write().await;
            if let Err(error) = doc.update(node.clone()).await {
                tracing::error!("While updating node: {error}");
            }
        }
    }

    /// An async background task that watches the document
    async fn watch_task(
        mut receiver: watch::Receiver<Node>,
        uri: Url,
        source: Arc<RwLock<String>>,
        root: Arc<RwLock<TextNode>>,
        mut client: ClientSocket,
    ) {
        while receiver.changed().await.is_ok() {
            let node = receiver.borrow_and_update().clone();

            // Encode the document to get generated content and mapping
            let (generated, EncodeInfo { mapping, .. }) = match codecs::to_string_with_info(
                &node,
                Some(EncodeOptions {
                    format: Some(Format::Markdown),
                    ..Default::default()
                }),
            )
            .await
            {
                Ok(node) => node,
                Err(error) => {
                    tracing::error!("While encoding document: {error}");
                    continue;
                }
            };

            // Walk the node to collect nodes and diagnostics
            let source = source.read().await;
            let mut inspector = Inspector::new(&source, &generated, mapping);
            inspector.visit(&node);

            // Publish diagnostics and update the root TextNode
            if let Some(text_node) = inspector.root() {
                diagnostics::publish(&uri, &text_node, &mut client);
                *root.write().await = text_node;
            }
        }
    }
}

/// Handle a notification from the client that a text document was opened
pub(super) fn did_open(
    state: &mut ServerState,
    params: DidOpenTextDocumentParams,
) -> ControlFlow<Result<(), Error>> {
    let uri = params.text_document.uri;
    let source = params.text_document.text;
    state.documents.insert(
        uri.clone(),
        TextDocument::new(uri, source, state.client.clone()),
    );

    ControlFlow::Continue(())
}

/// Handle a notification from the client that a text document was changes
pub(super) fn did_change(
    state: &mut ServerState,
    params: DidChangeTextDocumentParams,
) -> ControlFlow<Result<(), Error>> {
    let uri = params.text_document.uri;
    if let Some(doc) = state.documents.get_mut(&uri) {
        // TODO: This assumes a whole document change (with TextDocumentSyncKind::FULL in initialize):
        // needs more defensiveness and potentially implement incremental sync
        let source = params.content_changes[0].text.clone();
        if let Err(error) = doc.update_sender.send(source) {
            tracing::error!("While sending updated source: {error}");
        }
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
    state.documents.remove(&params.text_document.uri);

    ControlFlow::Continue(())
}
