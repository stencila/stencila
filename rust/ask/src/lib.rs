//! # Cross-Platform Confirmation Prompts
//!
//! This crate provides a unified abstraction for prompting users for
//! confirmation across different interfaces. The primary goal is to enable
//! library authors to write interface-agnostic code. For example, a file
//! manipulation library can ask for user confirmation before destructive
//! operations without needing to know whether it's being used in a CLI tool or
//! within a code editor via LSP.

use common::{
    async_trait::async_trait, eyre::Result, once_cell::sync::Lazy, strum::Display,
    tokio::sync::Mutex,
};

pub use crate::lsp::LspClient;
use crate::{cli::CliProvider, lsp::LspProvider};

mod cli;
mod lsp;

/// Setup with CLI provider
pub async fn setup_cli() -> Result<()> {
    global_context(AskContext::with_cli_provider()).await
}

/// Setup with LSP provider
pub async fn setup_lsp<C: LspClient + 'static>(client: C) -> Result<()> {
    global_context(AskContext::with_lsp_provider(client)).await
}

/// Ask a question
pub async fn ask(question: &str) -> Result<Answer> {
    ask_with(question, AskOptions::default()).await
}

/// Ask a question with default answer
pub async fn ask_with_default(question: &str, default: Answer) -> Result<Answer> {
    ask_with(
        question,
        AskOptions {
            default: Some(default),
            ..Default::default()
        },
    )
    .await
}

/// Ask a question with default answer and cancel allowed
pub async fn ask_with_default_and_cancel(question: &str, default: Answer) -> Result<Answer> {
    ask_with(
        question,
        AskOptions {
            default: Some(default),
            cancel_allowed: true,
            ..Default::default()
        },
    )
    .await
}

/// Ask a question with options
pub async fn ask_with(question: &str, options: AskOptions) -> Result<Answer> {
    let guard = GLOBAL_CONTEXT.lock().await;
    match guard.as_ref() {
        Some(ctx) => ctx.ask(question, options).await,
        None => {
            drop(guard);
            let ctx = AskContext::default();
            ctx.ask(question, options).await
        }
    }
}

/// Core trait that all confirmation providers must implement.
/// This abstraction allows different UI backends to provide user confirmation dialogs.
#[async_trait]
trait Ask: Send + Sync {
    /// Ask a question with additional customization options like custom button text,
    /// default selection, and whether cancellation is allowed.
    async fn ask(&self, question: &str, options: AskOptions) -> Result<Answer>;
}

/// Configuration options for customizing confirmation dialogs.
/// All fields are optional and providers should use sensible defaults when not specified.
#[derive(Default)]
pub struct AskOptions {
    /// Optional title for the dialog (only used for GUI/LSP contexts)
    pub title: Option<String>,

    /// Custom text for the "Yes" button (only used for GUI/LSP contexts)
    pub yes_text: Option<String>,

    /// Custom text for the "No" button (only used for GUI/LSP contexts)
    pub no_text: Option<String>,

    /// Default answer if the user just presses Enter (CLI) or closes the dialog (LSP)
    pub default: Option<Answer>,

    /// Whether the user can cancel/dismiss without answering
    pub cancel_allowed: bool,
}

/// The user's response to a question.
#[derive(Debug, Display, Clone, Copy, PartialEq, Eq)]
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

impl Default for AskContext {
    fn default() -> Self {
        // Always default to CLI, require explicit LSP setup
        Self::with_cli_provider()
    }
}

impl AskContext {
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

    pub async fn ask(&self, message: &str, options: AskOptions) -> Result<Answer> {
        self.provider.ask(message, options).await
    }
}

/// Global context
static GLOBAL_CONTEXT: Lazy<Mutex<Option<AskContext>>> = Lazy::new(|| Mutex::new(None));

/// Setup the global confirmation context
async fn global_context(context: AskContext) -> Result<()> {
    *GLOBAL_CONTEXT.lock().await = Some(context);
    Ok(())
}
