use async_lsp::ClientSocket;

mod initialize;
mod run;
pub use run::run;

/// The state of the language server
pub(crate) struct ServerState {
    /// The client of the language server
    /// 
    /// Used to communicate with the client e.g. send notifications.
    client: ClientSocket,
}
