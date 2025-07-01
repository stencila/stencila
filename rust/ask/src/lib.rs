//! # Cross-Platform Confirmation Prompts
//!
//! This crate provides a unified abstraction for prompting users for
//! confirmation across different interfaces. The primary goal is to enable
//! library authors to write interface-agnostic code. For example, a file
//! manipulation library can ask for user confirmation before destructive
//! operations without needing to know whether it's being used in a CLI tool or
//! within a code editor via LSP.
//!
//! ## Basic Usage
//!
//! ### CLI Applications
//!
//! ```rust
//! use ask::{setup_cli, ask, Answer};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Initialize for CLI usage
//!     setup_cli()?;
//!     
//!     match ask("Delete this file?").await? {
//!         Answer::Yes => println!("Deleting file..."),
//!         Answer::No => println!("File kept"),
//!         Answer::Cancel => println!("Operation cancelled"),
//!     }
//!     
//!     Ok(())
//! }
//! ```
//!
//! ### LSP Server Integration
//!
//! ```rust
//! use ask::{setup_lsp, ask, Answer, LspClient};
//! use common::{async_trait::async_trait, eyre::Result};
//!
//! struct MyLspClient {
//!     // LSP client implementation
//! }
//!
//! #[async_trait]
//! impl LspClient for MyLspClient {
//!     async fn show_message_request(
//!         &self,
//!         message: &str,
//!         actions: Vec<ask::MessageActionItem>
//!     ) -> Result<Option<ask::MessageActionItem>> {
//!         // Send window/showMessageRequest to LSP client
//!         todo!()
//!     }
//! }
//!
//! async fn handle_rename_request(client: MyLspClient) -> Result<()> {
//!     setup_lsp(client)?;
//!     
//!     match ask("Rename will affect 42 files. Continue?").await? {
//!         Answer::Yes => println!("Renaming files..."),
//!         Answer::No | Answer::Cancel => println!("Rename cancelled"),
//!     }
//!     
//!     Ok(())
//! }
//! ```
//!
//! ### Advanced Options
//!
//! ```rust
//! use ask::{ask_with_options, AskOptions, Answer};
//!
//! let options = AskOptions {
//!     title: Some("Destructive Operation".to_string()),
//!     default: Some(false), // Default to "No"
//!     yes_text: Some("Delete".to_string()),
//!     no_text: Some("Keep".to_string()),
//!     cancel_allowed: true,
//!     ..Default::default()
//! };
//!
//! match ask_with_options("Really delete all files?", options).await? {
//!     Answer::Yes => println!("Deleting..."),
//!     Answer::No => println!("Keeping files"),
//!     Answer::Cancel => println!("Cancelled"),
//! }
//! ```
use common::{async_trait::async_trait, eyre::Result, once_cell::sync::Lazy, tokio::sync::Mutex};

pub use crate::lsp::LspClient;
use crate::{cli::CliProvider, lsp::LspProvider};

mod cli;
mod lsp;

/// Initialize with CLI provider
pub async fn setup_cli() -> Result<()> {
    global_context(AskContext::with_cli_provider()).await
}

/// Initialize with LSP provider
pub async fn setup_lsp<C: LspClient + 'static>(client: C) -> Result<()> {
    global_context(AskContext::with_lsp_provider(client)).await
}

/// Ask a simple yes/no question using the global context
pub async fn ask(question: &str) -> Result<Answer> {
    let guard = GLOBAL_CONTEXT.lock().await;
    match guard.as_ref() {
        Some(ctx) => ctx.ask(question).await,
        None => {
            drop(guard);
            let ctx = AskContext::new();
            ctx.ask(question).await
        }
    }
}

/// Ask a question with options using the global context
pub async fn ask_with_options(question: &str, options: AskOptions) -> Result<Answer> {
    let guard = GLOBAL_CONTEXT.lock().await;
    match guard.as_ref() {
        Some(ctx) => ctx.ask_with_options(question, options).await,
        None => {
            drop(guard);
            let ctx = AskContext::new();
            ctx.ask_with_options(question, options).await
        }
    }
}

/// Core trait that all confirmation providers must implement.
/// This abstraction allows different UI backends to provide user confirmation dialogs.
#[async_trait]
trait Ask: Send + Sync {
    /// Ask a simple yes/no question and get the user's answer.
    async fn ask(&self, question: &str) -> Result<Answer>;

    /// Ask a question with additional customization options like custom button text,
    /// default selection, and whether cancellation is allowed.
    async fn ask_with_options(&self, question: &str, options: AskOptions) -> Result<Answer>;
}

/// Configuration options for customizing confirmation dialogs.
/// All fields are optional and providers should use sensible defaults when not specified.
#[derive(Default)]
pub struct AskOptions {
    /// Optional title for the dialog (mainly useful for GUI/LSP contexts)
    pub title: Option<String>,

    /// Default answer if the user just presses Enter (CLI) or closes the dialog (LSP)
    pub default: Option<bool>,

    /// Custom text for the "Yes" button/option
    pub yes_text: Option<String>,

    /// Custom text for the "No" button/option
    pub no_text: Option<String>,

    /// Whether the user can cancel/dismiss without answering
    pub cancel_allowed: bool,
}

/// The user's response to a question.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Answer {
    Yes,
    No,
    Cancel,
}

impl Answer {
    pub fn is_yes(&self) -> bool {
        matches!(self, Answer::Yes)
    }

    pub fn is_no(&self) -> bool {
        matches!(self, Answer::No)
    }

    pub fn is_cancel(&self) -> bool {
        matches!(self, Answer::Cancel)
    }

    pub fn is_no_or_cancel(&self) -> bool {
        matches!(self, Answer::No | Answer::Cancel)
    }
}

/// Context for managing providers
pub struct AskContext {
    provider: Box<dyn Ask>,
}

impl AskContext {
    pub fn new() -> Self {
        // Always default to CLI, require explicit LSP setup
        Self::with_cli_provider()
    }

    pub fn with_cli_provider() -> Self {
        Self {
            provider: Box::new(CliProvider),
        }
    }

    pub fn with_lsp_provider<C: LspClient + 'static>(client: C) -> Self {
        Self {
            provider: Box::new(LspProvider::new(client)),
        }
    }

    pub async fn ask(&self, message: &str) -> Result<Answer> {
        self.provider.ask(message).await
    }

    pub async fn ask_with_options(&self, message: &str, options: AskOptions) -> Result<Answer> {
        self.provider.ask_with_options(message, options).await
    }
}

/// Global context
static GLOBAL_CONTEXT: Lazy<Mutex<Option<AskContext>>> = Lazy::new(|| Mutex::new(None));

/// Initialize the global confirmation context
async fn global_context(context: AskContext) -> Result<()> {
    *GLOBAL_CONTEXT.lock().await = Some(context);
    Ok(())
}
