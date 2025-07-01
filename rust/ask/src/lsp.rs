use std::collections::HashMap;

use lsp_types::{MessageActionItem, MessageType, ShowMessageRequestParams};

use common::{async_trait::async_trait, eyre::Result};

use crate::{Answer, Ask, AskOptions};

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
    async fn ask(&self, question: &str) -> Result<Answer> {
        let params = ShowMessageRequestParams {
            typ: MessageType::INFO,
            message: question.into(),
            actions: Some(vec![
                MessageActionItem {
                    title: "Yes".to_string(),
                    properties: HashMap::new(),
                },
                MessageActionItem {
                    title: "No".to_string(),
                    properties: HashMap::new(),
                },
            ]),
        };

        let result = self.client.show_message_request(params).await?;

        match result {
            Some(action) if action.title == "Yes" => Ok(Answer::Yes),
            Some(_) => Ok(Answer::No),
            None => Ok(Answer::Cancel),
        }
    }

    async fn ask_with_options(&self, question: &str, options: AskOptions) -> Result<Answer> {
        let yes_text = options.yes_text.unwrap_or_else(|| "Yes".to_string());
        let no_text = options.no_text.unwrap_or_else(|| "No".to_string());

        let params = ShowMessageRequestParams {
            typ: MessageType::INFO,
            message: question.into(),
            actions: Some(vec![
                MessageActionItem {
                    title: yes_text.clone(),
                    properties: HashMap::new(),
                },
                MessageActionItem {
                    title: no_text.clone(),
                    properties: HashMap::new(),
                },
            ]),
        };

        let result = self.client.show_message_request(params).await?;

        match result {
            Some(action) if action.title == yes_text => Ok(Answer::Yes),
            Some(action) if action.title == no_text => Ok(Answer::No),
            Some(_) => Ok(Answer::No),
            None => {
                if options.cancel_allowed {
                    Ok(Answer::Cancel)
                } else {
                    Ok(Answer::No)
                }
            }
        }
    }
}
