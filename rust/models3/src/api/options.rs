use std::collections::HashMap;

use crate::client::Client;
use crate::error::SdkResult;
use crate::types::message::Message;
use crate::types::request::Request;
use crate::types::response_format::ResponseFormat;
use crate::types::timeout::Timeout;
use crate::types::tool::{ToolChoice, ToolDefinition};

/// The request-shaping fields shared across all Options structs.
///
/// Constructed once before the tool loop and reused each round with
/// fresh messages and tool definitions via [`to_request()`](Self::to_request).
#[derive(Debug, Clone)]
pub(crate) struct RequestTemplate {
    pub model: String,
    pub provider: Option<String>,
    pub tool_choice: Option<ToolChoice>,
    pub response_format: Option<ResponseFormat>,
    pub temperature: Option<f64>,
    pub top_p: Option<f64>,
    pub max_tokens: Option<u64>,
    pub stop_sequences: Option<Vec<String>>,
    pub reasoning_effort: Option<String>,
    pub provider_options: Option<HashMap<String, serde_json::Value>>,
    pub timeout: Option<Timeout>,
}

impl RequestTemplate {
    /// Build a [`Request`] from this template plus per-round messages and tools.
    pub fn to_request(
        &self,
        messages: &[Message],
        tool_defs: Option<&Vec<ToolDefinition>>,
    ) -> Request {
        Request {
            model: self.model.clone(),
            messages: messages.to_vec(),
            provider: self.provider.clone(),
            tools: tool_defs.cloned(),
            tool_choice: self.tool_choice.clone(),
            response_format: self.response_format.clone(),
            temperature: self.temperature,
            top_p: self.top_p,
            max_tokens: self.max_tokens,
            stop_sequences: self.stop_sequences.clone(),
            reasoning_effort: self.reasoning_effort.clone(),
            metadata: None,
            provider_options: self.provider_options.clone(),
            timeout: self.timeout,
        }
    }
}

/// Resolve the client from an `Option<&Client>`, falling back to the
/// default client when `None`.
pub(crate) fn resolve_client(client: Option<&Client>) -> SdkResult<&Client> {
    match client {
        Some(c) => Ok(c),
        None => super::default_client::get_default_client(),
    }
}

/// Generate the 12 builder methods common to all Options structs.
///
/// Expects the struct to have fields: `prompt`, `messages`, `system`,
/// `temperature`, `max_tokens`, `reasoning_effort`, `provider`,
/// `provider_options`, `max_retries`, `timeout`, `abort_signal`, `client`.
macro_rules! impl_common_builders {
    ($opts_type:ident<$lt:lifetime>) => {
        impl<$lt> $opts_type<$lt> {
            #[must_use]
            pub fn prompt(mut self, prompt: impl Into<String>) -> Self {
                self.prompt = Some(prompt.into());
                self
            }

            #[must_use]
            pub fn messages(mut self, messages: Vec<crate::types::message::Message>) -> Self {
                self.messages = messages;
                self
            }

            #[must_use]
            pub fn system(mut self, system: impl Into<String>) -> Self {
                self.system = Some(system.into());
                self
            }

            #[must_use]
            pub fn temperature(mut self, temp: f64) -> Self {
                self.temperature = Some(temp);
                self
            }

            #[must_use]
            pub fn max_tokens(mut self, tokens: u64) -> Self {
                self.max_tokens = Some(tokens);
                self
            }

            #[must_use]
            pub fn reasoning_effort(mut self, effort: impl Into<String>) -> Self {
                self.reasoning_effort = Some(effort.into());
                self
            }

            #[must_use]
            pub fn provider(mut self, provider: impl Into<String>) -> Self {
                self.provider = Some(provider.into());
                self
            }

            #[must_use]
            pub fn provider_options(
                mut self,
                options: std::collections::HashMap<String, serde_json::Value>,
            ) -> Self {
                self.provider_options = Some(options);
                self
            }

            #[must_use]
            pub fn max_retries(mut self, retries: u32) -> Self {
                self.max_retries = retries;
                self
            }

            #[must_use]
            pub fn timeout(mut self, timeout: crate::types::timeout::Timeout) -> Self {
                self.timeout = Some(timeout);
                self
            }

            #[must_use]
            pub fn abort_signal(mut self, signal: super::cancel::AbortSignal) -> Self {
                self.abort_signal = Some(signal);
                self
            }

            #[must_use]
            pub fn client(mut self, client: &$lt crate::client::Client) -> Self {
                self.client = Some(client);
                self
            }
        }
    };
}

pub(crate) use impl_common_builders;
