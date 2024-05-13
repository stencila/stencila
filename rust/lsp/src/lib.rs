use std::collections::HashMap;

use async_lsp::{lsp_types::Url, ClientSocket};

use common::serde::Deserialize;

mod code_lens;
mod commands;
mod completion;
mod content;
mod diagnostics;
mod formatting;
mod inspect;
mod lifecycle;
mod run;
mod symbols;
mod text_document;
mod utils;

pub use run::run;
use schema::Person;
use text_document::TextDocument;

/// The state of the language server
pub(crate) struct ServerState {
    /// The client of the language server
    ///
    /// Used to communicate with the client e.g. send notifications.
    client: ClientSocket,

    /// The configuration options defined in the client
    options: Option<ServerOptions>,

    /// The documents opened by the client that are handled by this server
    documents: HashMap<Url, TextDocument>,
}

#[derive(Deserialize)]
#[serde(crate = "common::serde")]
pub(crate) struct ServerOptions {
    user: Option<Person>,
}
