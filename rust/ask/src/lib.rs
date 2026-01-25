//! # Cross-Platform Confirmation Prompts
//!
//! This crate provides a unified abstraction for prompting users for
//! confirmation across different interfaces.
//!
//! The primary goal is to enable library authors to write interface-agnostic
//! code. For example, a file manipulation library can ask for user confirmation
//! before destructive operations without needing to know whether it's being
//! used in a CLI tool or within a code editor via LSP.

use std::sync::LazyLock;

use async_trait::async_trait;
use eyre::Result;
use strum::Display;
use tokio::sync::Mutex;

pub use crate::lsp::LspClient;
use crate::{cli::CliProvider, default::DefaultProvider, lsp::LspProvider};

mod cli;
mod default;
mod lsp;

/// Setup with CLI provider
pub async fn setup_cli(assume: Option<Answer>) -> Result<()> {
    global_context(AskContext::with_cli_provider(assume)).await
}

/// Setup with defaults provider (non-interactive, uses defaults for all prompts)
pub async fn setup_defaults() -> Result<()> {
    global_context(AskContext::default()).await
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
        Some(ctx) => {
            if let Some(answer) = ctx.assume {
                return Ok(answer);
            }
            ctx.ask(question, options).await
        }
        None => {
            drop(guard);
            let ctx = AskContext::default();
            ctx.ask(question, options).await
        }
    }
}

/// Ask for a password
pub async fn ask_for_password(prompt: &str) -> Result<String> {
    let guard = GLOBAL_CONTEXT.lock().await;
    match guard.as_ref() {
        Some(ctx) => ctx.password(prompt).await,
        None => {
            drop(guard);
            let ctx = AskContext::default();
            ctx.password(prompt).await
        }
    }
}

/// Prompt for text input
pub async fn input(prompt: &str) -> Result<String> {
    input_with(prompt, InputOptions::default()).await
}

/// Prompt for text input with a default value
pub async fn input_with_default(prompt: &str, default: &str) -> Result<String> {
    input_with(
        prompt,
        InputOptions {
            default: Some(default.to_string()),
            ..Default::default()
        },
    )
    .await
}

/// Prompt for text input with options
pub async fn input_with(prompt: &str, options: InputOptions) -> Result<String> {
    let guard = GLOBAL_CONTEXT.lock().await;
    match guard.as_ref() {
        Some(ctx) => ctx.input(prompt, options).await,
        None => {
            drop(guard);
            let ctx = AskContext::default();
            ctx.input(prompt, options).await
        }
    }
}

/// Prompt user to select one item from a list
pub async fn select(prompt: &str, items: &[String]) -> Result<usize> {
    select_with(prompt, items, SelectOptions::default()).await
}

/// Prompt user to select one item from a list with a default selection
pub async fn select_with_default(prompt: &str, items: &[String], default: usize) -> Result<usize> {
    select_with(
        prompt,
        items,
        SelectOptions {
            default: Some(default),
        },
    )
    .await
}

/// Prompt user to select one item from a list with options
pub async fn select_with(prompt: &str, items: &[String], options: SelectOptions) -> Result<usize> {
    let guard = GLOBAL_CONTEXT.lock().await;
    match guard.as_ref() {
        Some(ctx) => ctx.select(prompt, items, options).await,
        None => {
            drop(guard);
            let ctx = AskContext::default();
            ctx.select(prompt, items, options).await
        }
    }
}

/// Prompt user to select multiple items from a list
pub async fn multi_select(prompt: &str, items: &[String]) -> Result<Vec<usize>> {
    multi_select_with(prompt, items, MultiSelectOptions::default()).await
}

/// Prompt user to select multiple items from a list with default selections
pub async fn multi_select_with_defaults(
    prompt: &str,
    items: &[String],
    defaults: &[bool],
) -> Result<Vec<usize>> {
    multi_select_with(
        prompt,
        items,
        MultiSelectOptions {
            defaults: Some(defaults.to_vec()),
        },
    )
    .await
}

/// Prompt user to select multiple items from a list with options
pub async fn multi_select_with(
    prompt: &str,
    items: &[String],
    options: MultiSelectOptions,
) -> Result<Vec<usize>> {
    let guard = GLOBAL_CONTEXT.lock().await;
    match guard.as_ref() {
        Some(ctx) => ctx.multi_select(prompt, items, options).await,
        None => {
            drop(guard);
            let ctx = AskContext::default();
            ctx.multi_select(prompt, items, options).await
        }
    }
}

/// Wait for user to press Enter to continue
///
/// Use this when you need to pause execution until the user is ready to proceed,
/// such as after opening a browser window for authentication.
///
/// In non-interactive contexts (e.g., when stdin is not a TTY or when using the
/// default provider), this will return immediately without waiting.
pub async fn wait_for_enter(prompt: &str) -> Result<()> {
    let guard = GLOBAL_CONTEXT.lock().await;
    match guard.as_ref() {
        Some(ctx) => ctx.wait_for_enter(prompt).await,
        None => {
            drop(guard);
            let ctx = AskContext::default();
            ctx.wait_for_enter(prompt).await
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

    /// Prompt for a password.
    /// The password should be masked/hidden from display.
    async fn password(&self, prompt: &str) -> Result<String>;

    /// Prompt for text input.
    async fn input(&self, prompt: &str, options: InputOptions) -> Result<String>;

    /// Prompt user to select one item from a list.
    /// Returns the index of the selected item.
    async fn select(&self, prompt: &str, items: &[String], options: SelectOptions)
    -> Result<usize>;

    /// Prompt user to select multiple items from a list.
    /// Returns the indices of the selected items.
    async fn multi_select(
        &self,
        prompt: &str,
        items: &[String],
        options: MultiSelectOptions,
    ) -> Result<Vec<usize>>;

    /// Wait for the user to press Enter to continue.
    ///
    /// In non-interactive contexts, this should return immediately without waiting.
    async fn wait_for_enter(&self, prompt: &str) -> Result<()>;
}

/// Configuration options for customizing confirmation dialogs.
/// All fields are optional and providers should use sensible defaults when not specified.
#[derive(Default)]
pub struct AskOptions {
    /// The type of question being asked
    pub level: AskLevel,

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

impl AskOptions {
    /// Is a "Cancel" answer enabled?
    ///
    /// Returns `true` if `cancelled_allowed` or [`Answer::Cancel`] is the default
    pub fn cancel_enabled(&self) -> bool {
        self.cancel_allowed || matches!(self.default, Some(Answer::Cancel))
    }
}

/// Configuration options for text input prompts.
#[derive(Default)]
pub struct InputOptions {
    /// Default value to display and use if user presses Enter
    pub default: Option<String>,

    /// Whether to allow empty input
    pub allow_empty: bool,
}

/// Configuration options for single-selection prompts.
#[derive(Default)]
pub struct SelectOptions {
    /// Index of the default selection (0-based)
    pub default: Option<usize>,
}

/// Configuration options for multi-selection prompts.
#[derive(Default)]
pub struct MultiSelectOptions {
    /// Default selection state for each item (true = selected)
    pub defaults: Option<Vec<bool>>,
}

/// The type of question being asked
///
/// Mirrors logging levels
#[derive(Debug, Default, Display, Clone, Copy, PartialEq, Eq)]
pub enum AskLevel {
    #[default]
    Info,
    Warning,
    Error,
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
    /// The answer to assume
    assume: Option<Answer>,

    /// The asking provider
    provider: Box<dyn Ask>,
}

impl Default for AskContext {
    fn default() -> Self {
        Self {
            assume: None,
            provider: Box::new(DefaultProvider),
        }
    }
}

impl AskContext {
    pub fn with_cli_provider(assume: Option<Answer>) -> Self {
        if let Some(answer) = assume {
            tracing::debug!("Assuming answer `{answer}` for all interactive prompts");
        }

        Self {
            assume,
            provider: Box::new(CliProvider),
        }
    }

    pub fn with_lsp_provider<C: LspClient + 'static>(client: C) -> Self {
        Self {
            assume: None,
            provider: Box::new(LspProvider::new(client)),
        }
    }

    pub async fn ask(&self, message: &str, options: AskOptions) -> Result<Answer> {
        self.provider.ask(message, options).await
    }

    pub async fn password(&self, prompt: &str) -> Result<String> {
        self.provider.password(prompt).await
    }

    pub async fn input(&self, prompt: &str, options: InputOptions) -> Result<String> {
        self.provider.input(prompt, options).await
    }

    pub async fn select(
        &self,
        prompt: &str,
        items: &[String],
        options: SelectOptions,
    ) -> Result<usize> {
        self.provider.select(prompt, items, options).await
    }

    pub async fn multi_select(
        &self,
        prompt: &str,
        items: &[String],
        options: MultiSelectOptions,
    ) -> Result<Vec<usize>> {
        self.provider.multi_select(prompt, items, options).await
    }

    pub async fn wait_for_enter(&self, prompt: &str) -> Result<()> {
        self.provider.wait_for_enter(prompt).await
    }
}

/// Global context
static GLOBAL_CONTEXT: LazyLock<Mutex<Option<AskContext>>> = LazyLock::new(|| Mutex::new(None));

/// Setup the global confirmation context
async fn global_context(context: AskContext) -> Result<()> {
    *GLOBAL_CONTEXT.lock().await = Some(context);
    Ok(())
}
