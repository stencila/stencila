use std::collections::HashMap;

use async_trait::async_trait;
use eyre::{Result, bail};
use lsp_types::{MessageActionItem, MessageType, ShowMessageRequestParams};

use crate::{Answer, Ask, AskLevel, AskOptions, InputOptions, MultiSelectOptions, SelectOptions};

/// Trait for LSP client implementations
#[async_trait]
pub trait LspClient: Send + Sync {
    async fn show_message_request(
        &self,
        params: ShowMessageRequestParams,
    ) -> Result<Option<MessageActionItem>>;

    /// Request password input from the user
    ///
    /// This is not part of the standard LSP spec, but many LSP clients (like VS
    /// Code) support custom requests for input dialogs. Implementations should:
    ///
    /// - Use the client's password input capability if available (e.g.,
    ///   vscode.window.showInputBox with password: true)
    /// - Fall back to a regular input box with a warning if password input
    ///   isn't supported
    /// - Return an error if no input capability is available
    async fn request_password_input(&self, prompt: &str) -> Result<Option<String>>;
}

/// LSP confirmation provider
pub struct LspProvider<C: LspClient> {
    client: C,
}

impl<C: LspClient> LspProvider<C> {
    pub fn new(client: C) -> Self {
        Self { client }
    }
}

#[async_trait]
impl<C: LspClient> Ask for LspProvider<C> {
    async fn ask(&self, question: &str, options: AskOptions) -> Result<Answer> {
        let yes_text = options
            .yes_text
            .clone()
            .unwrap_or_else(|| "Yes".to_string());
        let no_text = options.no_text.clone().unwrap_or_else(|| "No".to_string());
        let cancel_text = "Cancel";

        let mut actions = vec![
            MessageActionItem {
                title: yes_text.clone(),
                properties: HashMap::new(),
            },
            MessageActionItem {
                title: no_text.clone(),
                properties: HashMap::new(),
            },
        ];
        if options.cancel_enabled() {
            actions.push(MessageActionItem {
                title: cancel_text.into(),
                properties: HashMap::new(),
            });
        }

        let params = ShowMessageRequestParams {
            typ: match options.level {
                AskLevel::Info => MessageType::INFO,
                AskLevel::Warning => MessageType::WARNING,
                AskLevel::Error => MessageType::ERROR,
            },
            message: question.into(),
            actions: Some(actions),
        };

        let result = self.client.show_message_request(params).await?;

        Ok(match result {
            Some(action) if action.title == yes_text => Answer::Yes,
            Some(action) if action.title == no_text => Answer::No,
            Some(action) if action.title == cancel_text => Answer::Cancel,
            Some(_) => Answer::No,
            None => {
                if options.cancel_enabled() {
                    Answer::Cancel
                } else {
                    Answer::No
                }
            }
        })
    }

    async fn password(&self, prompt: &str) -> Result<String> {
        match self.client.request_password_input(prompt).await {
            Ok(Some(password)) => Ok(password),
            Ok(None) => bail!("Password input cancelled by user"),
            Err(error) => {
                // If the client doesn't support password input, show a warning and explain the limitation
                let params = ShowMessageRequestParams {
                    typ: MessageType::ERROR,
                    message: format!(
                        "Password input is not supported by this editor: {error}\n\nPlease use the CLI interface for password-protected operations."
                    ),
                    actions: Some(vec![MessageActionItem {
                        title: "OK".to_string(),
                        properties: HashMap::new(),
                    }]),
                };

                self.client.show_message_request(params).await?;

                Err(error)
            }
        }
    }

    async fn input(&self, _prompt: &str, _options: InputOptions) -> Result<String> {
        bail!("Text input is not supported in LSP context. Please use the CLI instead.")
    }

    async fn select(
        &self,
        _prompt: &str,
        _items: &[String],
        _options: SelectOptions,
    ) -> Result<usize> {
        bail!("Selection is not supported in LSP context. Please use the CLI instead.")
    }

    async fn multi_select(
        &self,
        _prompt: &str,
        _items: &[String],
        _options: MultiSelectOptions,
    ) -> Result<Vec<usize>> {
        bail!("Multi-selection is not supported in LSP context. Please use the CLI instead.")
    }

    async fn wait_for_enter(&self, prompt: &str) -> Result<()> {
        let params = ShowMessageRequestParams {
            typ: MessageType::INFO,
            message: prompt.into(),
            actions: Some(vec![MessageActionItem {
                title: "OK".to_string(),
                properties: HashMap::new(),
            }]),
        };

        // Wait for user to click OK
        self.client.show_message_request(params).await?;

        Ok(())
    }
}
