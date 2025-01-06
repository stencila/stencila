#![recursion_limit = "256"]

use std::{collections::HashMap, env};

use async_lsp::{lsp_types::Url, ClientSocket};

use common::{eyre::Result, serde::Deserialize, serde_json, tracing};

mod code_lens;
mod commands;
mod completion;
mod content;
mod diagnostics;
mod dom;
mod formatting;
mod hover;
mod inspect;
mod kernels_;
mod lifecycle;
mod logging;
mod models_;
mod node_ids;
mod node_info;
mod prompts_;
mod run;
mod symbols;
mod text_document;
mod utils;

pub use run::run;
use schema::{Organization, Person};
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

    /// The status of the server
    status: ServerStatus,
}

pub(crate) enum ServerStatus {
    Running,
    Shutdown,
}

#[derive(Debug, Default, Deserialize)]
#[serde(crate = "common::serde")]
pub(crate) struct ServerOptions {
    /// The current user
    ///
    /// Used for attributing authorship.
    user: Option<ServerOptionsUser>,
}

impl ServerOptions {
    /// Initialize the options with a JSON value sent by the client with fallbacks to environment variables
    fn initialize(&mut self, value: serde_json::Value) -> Result<()> {
        *self = serde_json::from_value(value)?;

        if let Some(user) = &mut self.user {
            // User was initialized with settings
            user.initialize();
        } else {
            // Fallback to checking env
            let mut user = ServerOptionsUser::default();
            user.initialize();
            if user.object.is_some() {
                self.user = Some(user);
            }
        };

        tracing::trace!("Server options initialized: {self:?}");

        Ok(())
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(crate = "common::serde")]
pub(crate) struct ServerOptionsUser {
    /// The current user as a [`Person`] object
    object: Option<Person>,

    /// The current user's name
    ///
    /// Used as the user's `name` property, if not defined there
    name: Option<String>,

    /// The current user's affiliations
    ///
    /// Used to construct the `user` property if that is not provided
    affiliations: Option<Vec<String>>,
}

impl ServerOptionsUser {
    fn initialize(&mut self) {
        // If no `object` was set, attempt to set one from any STENCILA_USER env var
        if self.object.is_none() {
            if let Ok(value) = env::var("STENCILA_USER") {
                if let Ok(person) = serde_json::from_str(&value).or_else(|_| value.parse()) {
                    self.object = Some(person);
                }
            }
        }

        // Fill in properties that can be set individually
        if self.name.is_some() || self.affiliations.is_some() {
            let mut person = self.object.clone().unwrap_or_default();

            if let (true, Some(name)) = (person.options.name.is_none(), &self.name) {
                person.options.name = Some(name.clone());
            }

            if let (true, Some(affiliations)) = (person.affiliations.is_none(), &self.affiliations)
            {
                person.affiliations = Some(
                    affiliations
                        .iter()
                        .map(|name| Organization {
                            name: Some(name.clone()),
                            ..Default::default()
                        })
                        .collect(),
                );
            }

            self.object = Some(person);
        }
    }
}
