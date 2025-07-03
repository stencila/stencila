use std::collections::HashMap;

use lsp_types::{MessageActionItem, MessageType, ShowMessageRequestParams};

use common::{async_trait::async_trait, eyre::Result};

use crate::{Answer, Ask, AskLevel, AskOptions};

/// Trait for LSP client implementations
#[async_trait]
pub trait LspClient: Send + Sync {
    async fn show_message_request(
        &self,
        params: ShowMessageRequestParams,
    ) -> Result<Option<MessageActionItem>>;
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

    async fn password(&self, _prompt: &str) -> Result<String> {
        todo!("Password input via LSP is not yet implemented")
    }
}
