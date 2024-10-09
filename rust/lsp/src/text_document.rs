//! Handling of text document synchronization related messages
//!
//! See https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_synchronization

use core::time;
use std::{ops::ControlFlow, path::PathBuf, sync::Arc};

use async_lsp::{
    lsp_types::{
        Diagnostic, DiagnosticSeverity, DidChangeTextDocumentParams, DidCloseTextDocumentParams,
        DidOpenTextDocumentParams, DidSaveTextDocumentParams, MessageType, Position,
        PublishDiagnosticsParams, Range, ShowMessageParams, Url,
    },
    ClientSocket, Error, ErrorCode, LanguageClient, ResponseError,
};

use codecs::{
    DecodeInfo, DecodeOptions, EncodeInfo, EncodeOptions, Format, LossesResponse, MessageLevel,
    Messages,
};
use common::{
    eyre::{bail, Report},
    tokio::{
        self,
        sync::{mpsc, watch, RwLock},
    },
    tracing,
};
use document::{CommandWait, Document};
use schema::{
    Author, AuthorRole, AuthorRoleName, Duration, ExecutionKind, ExecutionMessage, ExecutionMode,
    ExecutionRequired, ExecutionStatus, Node, NodeId, NodeType, Person, ProvenanceCount, Timestamp,
    Visitor,
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

    /// The type of the parent of the node
    pub parent_type: NodeType,

    /// The id of the parent of the node
    #[allow(unused)]
    pub parent_id: NodeId,

    /// The type of the node
    pub node_type: NodeType,

    /// The id of the node
    pub node_id: NodeId,

    /// A string name of the node
    ///
    /// Used when creating a document symbol for the node.
    /// Defaults to the stringified node type.
    pub name: String,

    /// A string detail of the node
    ///
    /// Used when creating a document symbol for the node.
    pub detail: Option<String>,

    /// Execution details (for executable nodes only)
    ///
    /// These detail are used to publish diagnostics and status
    /// notifications for the node
    pub execution: Option<TextNodeExecution>,

    /// Whether the node is active (currently for `IfBlockClause` nodes only)
    pub is_active: Option<bool>,

    /// Provenance details (for nodes with a `provenance` field)
    ///
    /// These detail are used to publish provenance summaries
    /// for the node
    pub provenance: Option<Vec<ProvenanceCount>>,

    /// The children of the node
    pub children: Vec<TextNode>,
}

#[allow(unused)]
#[derive(Debug, Clone, Default)]
pub(super) struct TextNodeExecution {
    pub mode: Option<ExecutionMode>,
    pub status: Option<ExecutionStatus>,
    pub required: Option<ExecutionRequired>,
    pub kind: Option<ExecutionKind>,
    pub duration: Option<Duration>,
    pub ended: Option<Timestamp>,
    pub outputs: Option<usize>,
    pub messages: Option<Vec<ExecutionMessage>>,
    pub authors: Option<Vec<Author>>,
}

impl Default for TextNode {
    fn default() -> Self {
        Self {
            range: Range::default(),
            parent_type: NodeType::Null,
            parent_id: NodeId::null(),
            node_type: NodeType::Null,
            node_id: NodeId::null(),
            name: String::new(),
            detail: None,
            execution: None,
            is_active: None,
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
    /// Get the node id at a position (if any)
    pub fn node_id_at(&self, position: Position) -> Option<NodeId> {
        // Search through children (and thus recursively through all
        // descendants so that the deepest (most narrow range) node is selected)
        for child in &self.children {
            if let Some(node_id) = child.node_id_at(position) {
                return Some(node_id);
            }
        }

        // If no descendants in range then check if this is
        if position >= self.range.start && position < self.range.end {
            return Some(self.node_id.clone());
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
    /// The author of the document
    pub author: AuthorRole,

    /// The format of the document
    pub format: Format,

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
    fn new(
        uri: Url,
        format: String,
        source: String,
        client: ClientSocket,
        user: Option<Person>,
    ) -> Result<Self, Report> {
        // Get path without percent encodings (e.g. for spaces)
        let path = percent_encoding::percent_decode_str(uri.path()).decode_utf8_lossy();

        let path = PathBuf::from(path.as_ref());
        let Some(home) = path.parent() else {
            bail!("File does not have a parent dir")
        };
        let doc = Document::init(home.into(), Some(path))?;

        let format = Format::from_name(&format);

        let person = user.unwrap_or_else(|| Person {
            given_names: Some(vec!["Anonymous".to_string()]),
            ..Default::default()
        });
        let author = AuthorRole {
            author: schema::AuthorRoleAuthor::Person(person),
            role_name: AuthorRoleName::Writer,
            format: Some(format.name().to_string()),
            ..Default::default()
        };

        let watch_receiver = doc.watch();

        let source_string = source.clone();

        let source = Arc::new(RwLock::new(source));
        let root = Arc::new(RwLock::new(TextNode::default()));
        let doc = Arc::new(RwLock::new(doc));

        let (update_sender, update_receiver) = mpsc::unbounded_channel();

        {
            let uri = uri.clone();
            let format = format.clone();
            let source = source.clone();
            let doc = doc.clone();
            let author = author.clone();
            let client = client.clone();
            tokio::spawn(async {
                Self::update_task(update_receiver, uri, format, source, doc, author, client).await;
            });
        }

        {
            let format = format.clone();
            let source = source.clone();
            let root = root.clone();
            tokio::spawn(async move {
                Self::watch_task(watch_receiver, uri, format, source, root, client).await;
            });
        }

        if let Err(error) = update_sender.send(source_string) {
            tracing::error!("While sending initial source: {error}");
        }

        Ok(TextDocument {
            author,
            format,
            source,
            root,
            doc,
            update_sender,
        })
    }

    /// An async background task which updates the source and
    /// the Stencila document when there change in the editor
    ///
    /// Uses debouncing for two reasons:
    ///
    /// - so that edits to the document end up being applied a replace patches
    ///   rather than a series of single character additions and deletions, there
    ///   is a tradeoff here between granularity and latency
    ///
    /// - to avoid excessive compute decoding the document on each keypress
    async fn update_task(
        mut receiver: mpsc::UnboundedReceiver<String>,
        uri: Url,
        format: Format,
        source: Arc<RwLock<String>>,
        doc: Arc<RwLock<Document>>,
        author_role: AuthorRole,
        client: ClientSocket,
    ) {
        // As a guide, average typing speed is around 200 chars per minute which means
        // 60000 / 200 = 300 milliseconds per character.
        const DEBOUNCE_DELAY_MILLIS: u64 = 500;
        let debounce = time::Duration::from_millis(DEBOUNCE_DELAY_MILLIS);

        // Spawn a task to publish diagnostics related to decoding the source
        let (messages_sender, messages_receiver) = mpsc::channel(24);
        tokio::spawn(async move { Self::diagnostics_task(messages_receiver, uri, client).await });

        let mut latest_source = None;
        loop {
            // Debounce updates
            let new_source = match tokio::time::timeout(debounce, receiver.recv()).await {
                Ok(None) => {
                    // Received nothing: sender has dropped so stop
                    break;
                }
                Ok(Some(source)) => {
                    // Received new source: update the latest source
                    latest_source = Some(source);
                    continue;
                }
                Err(..) => {
                    // Timeout: if no new source since last timeout then continue
                    // otherwise proceed with below. Note that `take()` will `None`ify
                    // the `latest_source`
                    let Some(source) = latest_source.take() else {
                        continue;
                    };
                    source
                }
            };

            // Update the source
            *source.write().await = new_source.clone();

            // Decode the source into a node
            let (node, DecodeInfo { messages, .. }) = match codecs::from_str_with_info(
                &new_source,
                Some(DecodeOptions {
                    format: Some(format.clone()),
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
            let errors = messages
                .iter()
                .any(|message| matches!(message.level, MessageLevel::Error));

            // Always send messages, even if empty (to clear any previous diagnostics)
            if let Err(error) = messages_sender.send(messages).await {
                tracing::error!("While sending decoding messages: {error}");
            };

            // If there are any errors while decoding the source ignore this update
            if errors {
                continue;
            }

            // Update the Stencila document with the new node
            let doc = doc.write().await;
            if let Err(error) = doc
                .update(
                    node.clone(),
                    Some(Format::Markdown),
                    Some(vec![author_role.clone()]),
                )
                .await
            {
                tracing::error!("While updating node: {error}");
            }
        }
    }

    /// An async background task to publish diagnostics related to decoding the document
    async fn diagnostics_task(
        mut receiver: mpsc::Receiver<Messages>,
        uri: Url,
        mut client: ClientSocket,
    ) {
        // This is the delay before publishing diagnostics. It is additional to the delay in processing
        // updates to source and is here so that the user has a chance to write valid syntax before getting
        // warnings and errors related to incomplete syntax
        const DEBOUNCE_DELAY_MILLIS: u64 = 1000;
        let debounce = time::Duration::from_millis(DEBOUNCE_DELAY_MILLIS);

        let mut latest_messages = None;
        loop {
            // Debounce updates
            let messages = match tokio::time::timeout(debounce, receiver.recv()).await {
                Ok(None) => {
                    // Received nothing: sender has dropped so stop this task
                    break;
                }
                Ok(Some(new_messages)) => {
                    // Received new messages: if not empty then continue to wait for timeout
                    // otherwise proceed with below so messages get cleared
                    if new_messages.is_empty() {
                        latest_messages = None;
                        new_messages
                    } else {
                        latest_messages = Some(new_messages);
                        continue;
                    }
                }
                Err(..) => {
                    // Timeout: if no new messages since last timeout then continue
                    // otherwise proceed with below. Note that `take()` will `None`ify
                    // the `messages`
                    let Some(messages) = latest_messages.take() else {
                        continue;
                    };
                    messages
                }
            };

            let mut diagnostics = Vec::new();
            for message in messages.0 {
                let severity = Some(match message.level {
                    MessageLevel::Debug | MessageLevel::Trace => DiagnosticSeverity::HINT,
                    MessageLevel::Info => DiagnosticSeverity::INFORMATION,
                    MessageLevel::Warning => DiagnosticSeverity::WARNING,
                    MessageLevel::Error => DiagnosticSeverity::ERROR,
                });
                let position = Position {
                    line: message.start_line.unwrap_or_default() as u32,
                    character: 0,
                };
                diagnostics.push(Diagnostic {
                    severity,
                    message: message.message,
                    range: Range {
                        start: position,
                        end: position,
                    },
                    ..Default::default()
                })
            }
            if let Err(error) = client.publish_diagnostics(PublishDiagnosticsParams {
                uri: uri.clone(),
                diagnostics,
                version: None,
            }) {
                tracing::error!("While publishing diagnostics: {error}");
            }
        }
    }

    /// An async background task that watches the document
    async fn watch_task(
        mut receiver: watch::Receiver<Node>,
        uri: Url,
        format: Format,
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
                    format: Some(format.clone()),
                    // Reduce log level for reporting encoding losses
                    losses: LossesResponse::Trace,
                    ..Default::default()
                }),
            )
            .await
            {
                Ok(result) => result,
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
                //eprintln!("ROOT: {text_node:#?}");
                diagnostics::publish(&uri, &text_node, &mut client);
                *root.write().await = text_node;
            }

            // Ask the client to refresh code lenses. This is important for things
            // like provenance statistics code lenses which should be updated on each
            // update to the document
            client.code_lens_refresh(()).await.ok();
        }
    }
}

/// Handle a notification from the client that a text document was opened
pub(super) fn did_open(
    state: &mut ServerState,
    params: DidOpenTextDocumentParams,
) -> ControlFlow<Result<(), Error>> {
    let uri = params.text_document.uri;
    let format = params.text_document.language_id;
    let source = params.text_document.text;

    let client = state.client.clone();
    let user = state
        .options
        .user
        .as_ref()
        .and_then(|user| user.object.clone());

    let text_doc = match TextDocument::new(uri.clone(), format, source, client, user) {
        Ok(doc) => doc,
        Err(error) => {
            return ControlFlow::Break(Err(Error::Response(ResponseError::new(
                ErrorCode::INTERNAL_ERROR,
                format!("While creating new document: {error}"),
            ))))
        }
    };

    state.documents.insert(uri, text_doc);

    ControlFlow::Continue(())
}

/// Handle a notification from the client that a text document was changes
pub(super) fn did_change(
    state: &mut ServerState,
    params: DidChangeTextDocumentParams,
) -> ControlFlow<Result<(), Error>> {
    let uri = params.text_document.uri;
    if let Some(text_doc) = state.documents.get_mut(&uri) {
        // TODO: This assumes a whole document change (with TextDocumentSyncKind::FULL in initialize):
        // needs more defensiveness and potentially implement incremental sync
        let source = params.content_changes[0].text.clone();
        if let Err(error) = text_doc.update_sender.send(source) {
            tracing::error!("While sending updated source: {error}");
        }
    } else {
        tracing::warn!("Unknown document `${uri}`")
    }

    ControlFlow::Continue(())
}

/// Handle a notification from the client that a text document was save
///
/// Saves the document's sidecar file to disk.
pub(super) fn did_save(
    state: &mut ServerState,
    params: DidSaveTextDocumentParams,
) -> ControlFlow<Result<(), Error>> {
    if let Some(text_doc) = state.documents.get(&params.text_document.uri) {
        let doc = text_doc.doc.clone();
        let mut client = state.client.clone();
        tokio::spawn(async move {
            let doc = doc.read().await;
            if let Err(error) = doc.save(CommandWait::Yes).await {
                client
                    .show_message(ShowMessageParams {
                        typ: MessageType::ERROR,
                        message: format!("Error saving document: {error}"),
                    })
                    .ok();
            }
        });
    }

    ControlFlow::Continue(())
}

/// Handle a notification from the client that a text document was closed
///
/// Removes the document from the server state's list of documents.
/// Does NOT save the document because the editor (e.g. VSCode) should
/// have prompted the user asking if they wanted to save any changes already
/// (and may have said no to saving unsaved changes).
pub(super) fn did_close(
    state: &mut ServerState,
    params: DidCloseTextDocumentParams,
) -> ControlFlow<Result<(), Error>> {
    state.documents.remove(&params.text_document.uri);

    ControlFlow::Continue(())
}
