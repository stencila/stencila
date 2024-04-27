use std::collections::HashMap;

use async_lsp::ClientSocket;

use document::Document;

mod lifecycle;
mod run;
mod text_document;

pub use run::run;

/// The state of the language server
#[derive(Debug)]
pub(crate) struct ServerState {
    /// The client of the language server
    ///
    /// Used to communicate with the client e.g. send notifications.
    client: ClientSocket,

    /// The documents opened by the client that are handled by this server
    documents: HashMap<String, Document>,
}
