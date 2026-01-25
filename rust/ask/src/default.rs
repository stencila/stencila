use async_trait::async_trait;
use eyre::{Result, bail};

use crate::{Answer, Ask, AskLevel, AskOptions, InputOptions, MultiSelectOptions, SelectOptions};

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

    async fn input(&self, prompt: &str, options: InputOptions) -> Result<String> {
        let value = options.default.unwrap_or_default();
        tracing::info!("{prompt} -> Auto-selected: `{value}`");
        Ok(value)
    }

    async fn select(
        &self,
        prompt: &str,
        items: &[String],
        options: SelectOptions,
    ) -> Result<usize> {
        // Guard against out-of-range default index
        let idx = options
            .default
            .unwrap_or(0)
            .min(items.len().saturating_sub(1));
        let selected = items.get(idx).map(String::as_str).unwrap_or("");
        tracing::info!("{prompt} -> Auto-selected: `{selected}`");
        Ok(idx)
    }

    async fn multi_select(
        &self,
        prompt: &str,
        items: &[String],
        options: MultiSelectOptions,
    ) -> Result<Vec<usize>> {
        // If defaults provided, use those; otherwise select the first item
        // Guard against out-of-range indices by clamping to items.len()
        let selections: Vec<usize> = options
            .defaults
            .map(|d| {
                d.iter()
                    .enumerate()
                    .take(items.len()) // Ignore defaults beyond items length
                    .filter_map(|(i, &v)| if v { Some(i) } else { None })
                    .collect()
            })
            .unwrap_or_else(|| if items.is_empty() { vec![] } else { vec![0] });

        let selected: Vec<&str> = selections
            .iter()
            .filter_map(|&i| items.get(i).map(String::as_str))
            .collect();
        tracing::info!("{prompt} -> Auto-selected: {:?}", selected);

        Ok(selections)
    }

    async fn wait_for_enter(&self, prompt: &str) -> Result<()> {
        tracing::debug!("{prompt} (skipped, non-interactive)");
        Ok(())
    }
}
