use std::{collections::HashMap, env};

use async_lsp::{lsp_types::Url, ClientSocket};

use common::{eyre::Result, serde::Deserialize, serde_json, tracing};

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

    /// The configuration options defined by the client or using environment variables
    options: ServerOptions,

    /// The documents opened by the client that are handled by this server
    documents: HashMap<Url, TextDocument>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(crate = "common::serde")]
pub(crate) struct ServerOptions {
    /// The current user
    ///
    /// Used for attributing authorship.
    user: Option<Person>,
}

impl ServerOptions {
    /// Initialize the options with a JSON value sent by the client with fallbacks to environment variables
    fn initialize(&mut self, value: serde_json::Value) -> Result<()> {
        *self = serde_json::from_value(value)?;

        // If not user was set, attempt to set one from any STENCILA_USER env var
        if self.user.is_none() {
            if let Ok(value) = env::var("STENCILA_USER") {
                tracing::debug!("3 {value}");
                if let Ok(person) = serde_json::from_str(&value).or_else(|_| value.parse()) {
                    self.user = Some(person);
                }
            }
        }

        tracing::debug!("Server options initialized: {self:?}");

        Ok(())
    }
}
