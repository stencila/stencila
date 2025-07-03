use common::{
    async_trait::async_trait,
    eyre::{Result, bail},
    tracing,
};

use crate::{Answer, Ask, AskLevel, AskOptions};

/// Default provider
///
/// Answers with the default answer, or [`Answer::Yes`] if there is no default,
/// and logs the question and that answer.
/// Used when no other provider is setup, such as when Stencila Rust is being
/// called from Stencila's Python or Node.js bindings.
pub struct DefaultProvider;

#[async_trait]
impl Ask for DefaultProvider {
    async fn ask(&self, question: &str, options: AskOptions) -> Result<Answer> {
        let answer = options.default.unwrap_or(Answer::Yes);

        let message = format!("{question} Defaulting to answering `{answer}`");
        match options.level {
            AskLevel::Info => tracing::info!("{message}"),
            AskLevel::Warning => tracing::warn!("{message}"),
            AskLevel::Error => tracing::error!("{message}"),
        }

        Ok(answer)
    }

    async fn password(&self, _prompt: &str) -> Result<String> {
        bail!("Password input is not available in non-interactive contexts")
    }
}
