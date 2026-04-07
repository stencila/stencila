use std::path::Path;

use stencila_linter::{
    Format, Linter, LinterAvailability, LintingOptions, LintingOutput, NodeType, async_trait,
    eyre::Result,
    stencila_schema::{CompilationMessage, MessageLevel},
};

#[derive(Default)]
pub struct SvgLint;

#[async_trait]
impl Linter for SvgLint {
    fn name(&self) -> &str {
        "svglint"
    }

    fn node_types(&self) -> Vec<NodeType> {
        vec![NodeType::Figure, NodeType::CodeChunk]
    }

    fn formats(&self) -> Vec<Format> {
        vec![Format::Svg]
    }

    fn supports_formatting(&self) -> bool {
        false
    }

    fn supports_fixing(&self) -> bool {
        false
    }

    fn availability(&self) -> LinterAvailability {
        LinterAvailability::Available
    }

    #[tracing::instrument(skip(self))]
    async fn lint(
        &self,
        content: &str,
        _path: &Path,
        _options: &LintingOptions,
    ) -> Result<LintingOutput> {
        tracing::trace!("Linting SVG");

        // Currently only does linting from SVG components crate but in the future
        // could do other forms of SVG linting
        let result = stencila_svg_components::lint(content);

        if result.messages.is_empty() {
            return Ok(LintingOutput::default());
        }

        let messages: Vec<CompilationMessage> = result
            .messages
            .into_iter()
            .map(|m| {
                let level = match m.level {
                    stencila_svg_components::diagnostics::MessageLevel::Error => {
                        MessageLevel::Error
                    }
                    stencila_svg_components::diagnostics::MessageLevel::Warning => {
                        MessageLevel::Warning
                    }
                };
                CompilationMessage {
                    level,
                    message: m.message,
                    error_type: Some("SvgLint".to_string()),
                    ..Default::default()
                }
            })
            .collect();

        Ok(LintingOutput {
            messages: Some(messages),
            ..Default::default()
        })
    }
}
