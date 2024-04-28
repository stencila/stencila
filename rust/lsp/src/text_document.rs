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
    pub nodes: Arc<RwLock<Vec<(Range, NodeType, NodeId)>>>,
}

impl TextDocument {
    /// Create a new text document with an initial source
    fn new(source: String) -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        let nodes = Arc::new(RwLock::default());

        let nodes_clone = nodes.clone();
        tokio::spawn(async {
            Self::update_task(receiver, nodes_clone).await;
        });

        if let Err(error) = sender.send(source) {
            tracing::error!("While sending initial source: {error}");
        }

        Self { sender, nodes }
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
        nodes: Arc<RwLock<Vec<(Range, NodeType, NodeId)>>>,
    ) {
        while let Some(source) = receiver.recv().await {
            // Take a write lock on nodes so that readers can not read
            // until the update is finished
            let mut nodes = nodes.write().await;

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
            *nodes = inspector.nodes;
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
