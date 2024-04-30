use std::collections::HashMap;

use async_lsp::ClientSocket;

mod code_lens;
mod commands;
mod document_symbol;
mod inspect;
mod lifecycle;
mod run;
mod text_document;

pub use run::run;
use text_document::TextDocument;

/// The state of the language server
pub(crate) struct ServerState {
    /// The client of the language server
    ///
    /// Used to communicate with the client e.g. send notifications.
    client: ClientSocket,

    /// The documents opened by the client that are handled by this server
    documents: HashMap<String, TextDocument>,
}
