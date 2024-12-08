//! Handling of notebook document synchronization related messages
//!
//! See https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#notebookDocument_synchronization

use std::{ops::ControlFlow, path::PathBuf, sync::Arc};

use async_lsp::{
    lsp_types::{
        DidChangeNotebookDocumentParams, DidCloseNotebookDocumentParams,
        DidOpenNotebookDocumentParams, DidSaveNotebookDocumentParams, NotebookCell,
        NotebookCellKind, NotebookDocumentChangeEvent, TextDocumentItem, Uri,
    },
    Error, ErrorCode, ResponseError,
};

use codecs::Format;
use common::{
    eyre::{bail, Report},
    tokio::{
        self,
        sync::{mpsc, RwLock},
    },
    tracing,
};
use document::Document;
use schema::{Article, Author, AuthorRole, AuthorRoleName, Node, Person};

use crate::ServerState;

/// A notebook document that has been opened by the language server
pub(super) struct NotebookDocument {
    /// The cells of the notebook
    pub cells: Arc<RwLock<Vec<NotebookCell>>>,

    /// The text documents associated with each cell
    pub texts: Arc<RwLock<Vec<TextDocumentItem>>>,

    /// The Stencila document for the notebook document
    pub doc: Arc<RwLock<Document>>,

    /// A sender to the `update_task`
    ///
    /// Sends change events to the `update_task`. This is an `UnboundedSender`
    /// so that updates can be sent from sync functions
    update_sender: mpsc::UnboundedSender<NotebookDocumentChangeEvent>,
}

impl NotebookDocument {
    /// Create a new text document with an initial set of cells
    fn new(
        uri: Uri,
        cells: Vec<NotebookCell>,
        texts: Vec<TextDocumentItem>,
        user: Option<Person>,
    ) -> Result<Self, Report> {
        // Get path without percent encodings (e.g. for spaces)
        let path = percent_encoding::percent_decode_str(uri.path().as_str()).decode_utf8_lossy();

        let path = PathBuf::from(path.as_ref());
        let format = Format::from_path(&path);
        let Some(home) = path.parent() else {
            bail!("File does not have a parent dir")
        };

        let person = user.unwrap_or_else(|| Person {
            given_names: Some(vec!["Anonymous".to_string()]),
            ..Default::default()
        });
        let author_role = AuthorRole {
            author: schema::AuthorRoleAuthor::Person(person),
            role_name: AuthorRoleName::Writer,
            format: Some(format.name().to_string()),
            ..Default::default()
        };

        let doc = Document::init(home.into(), Some(path))?;
        let article = Self::decode_article(&cells, &texts);

        let cells = Arc::new(RwLock::new(cells));
        let texts = Arc::new(RwLock::new(texts));
        let doc = Arc::new(RwLock::new(doc));

        let (update_sender, update_receiver) = mpsc::unbounded_channel();
        {
            let cells = cells.clone();
            let texts = texts.clone();
            let doc = doc.clone();
            tokio::spawn(async {
                Self::update_task(update_receiver, cells, texts, doc, author_role).await;
            });
        }

        Ok(NotebookDocument {
            cells,
            texts,
            doc,
            update_sender,
        })
    }

    /// An async background task which updates the source and
    /// the Stencila document when there change in the editor
    async fn update_task(
        mut receiver: mpsc::UnboundedReceiver<NotebookDocumentChangeEvent>,
        cells: Arc<RwLock<Vec<NotebookCell>>>,
        docs: Arc<RwLock<Vec<TextDocumentItem>>>,
        doc: Arc<RwLock<Document>>,
        author_role: AuthorRole,
    ) {
        while let Some(change_event) = receiver.recv().await {
            tracing::debug!("Received notebook change event");

            // TODO: update cells, translate to Stencila document and update it
        }
    }

    fn decode_article(cells: &[NotebookCell], docs: &[TextDocumentItem]) -> Node {
        Node::Article(Article::default())
    }
}

/// Handle a notification from the client that a text document was opened
pub(super) fn did_open(
    state: &mut ServerState,
    params: DidOpenNotebookDocumentParams,
) -> ControlFlow<Result<(), Error>> {
    let uri = params.notebook_document.uri;
    let cells = params.notebook_document.cells;
    let cell_docs = params.cell_text_documents;

    tracing::debug!(
        "Opening notebook {} with {} cells",
        uri.to_string(),
        cells.len()
    );

    let user = state
        .options
        .user
        .as_ref()
        .and_then(|user| user.object.clone());

    let doc = match NotebookDocument::new(uri.clone(), cells, cell_docs, user) {
        Ok(doc) => doc,
        Err(error) => {
            return ControlFlow::Break(Err(Error::Response(ResponseError::new(
                ErrorCode::INTERNAL_ERROR,
                format!("While creating new document: {error}"),
            ))))
        }
    };

    state.notebook_documents.insert(uri, doc);

    ControlFlow::Continue(())
}

/// Handle a notification from the client that a notebook document was changed
pub(super) fn did_change(
    state: &mut ServerState,
    params: DidChangeNotebookDocumentParams,
) -> ControlFlow<Result<(), Error>> {
    let uri = params.notebook_document.uri;
    if let Some(doc) = state.notebook_documents.get_mut(&uri) {
        let change = params.change;
        if let Err(error) = doc.update_sender.send(change) {
            tracing::error!("While sending updated source: {error}");
        }
    } else {
        tracing::warn!("Unknown document `{}`", uri.to_string())
    }

    ControlFlow::Continue(())
}

/// Handle a notification from the client that a notebook document was saved
pub(super) fn did_save(
    _state: &mut ServerState,
    params: DidSaveNotebookDocumentParams,
) -> ControlFlow<Result<(), Error>> {
    let uri = params.notebook_document.uri;
    tracing::debug!("Saving notebook {}", uri.to_string());

    ControlFlow::Continue(())
}

/// Handle a notification from the client that a notebook document was closed
pub(super) fn did_close(
    state: &mut ServerState,
    params: DidCloseNotebookDocumentParams,
) -> ControlFlow<Result<(), Error>> {
    let uri = params.notebook_document.uri;
    tracing::debug!("Closing notebook {}", uri.to_string());

    state.notebook_documents.remove(&uri);

    ControlFlow::Continue(())
}
