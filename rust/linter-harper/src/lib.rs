use std::path::Path;

use harper_core::{
    Dialect, Document,
    linting::{Lint, LintGroup, LintKind, Linter as _, Suggestion},
    spell::FstDictionary,
};
use stencila_linter::{
    Format, Linter, LinterAvailability, LintingOptions, LintingOutput, NodeType,
    common::{
        async_trait::async_trait, eyre::Result, itertools::Itertools, once_cell::sync::Lazy,
        tokio::sync::Mutex, tracing,
    },
    schema::{
        AuthorRoleName, CodeLocation, CompilationMessage, MessageLevel, SoftwareApplication,
        Timestamp,
    },
};
#[derive(Default)]
pub struct HarperLinter;

#[async_trait]
impl Linter for HarperLinter {
    fn name(&self) -> &str {
        "harper"
    }

    fn node_types(&self) -> Vec<NodeType> {
        vec![NodeType::Text]
    }

    fn formats(&self) -> Vec<Format> {
        vec![Format::Text]
    }

    fn availability(&self) -> LinterAvailability {
        LinterAvailability::Available
    }

    #[tracing::instrument(skip(self, text))]
    async fn lint(
        &self,
        text: &str,
        path: &Path,
        options: &LintingOptions,
    ) -> Result<LintingOutput> {
        tracing::trace!("Linting with Harper");

        static LINTER: Lazy<Mutex<LintGroup>> = Lazy::new(|| {
            Mutex::new(LintGroup::new_curated(
                FstDictionary::curated(),
                Dialect::American,
            ))
        });

        let document = Document::new_plain_english_curated(text);
        let lints = LINTER.lock().await.lint(&document);

        let (new_text, messages) = if options.should_format || options.should_fix {
            // Apply Harper's suggested fixes for each lint.
            // Harper offers multiple suggestions for each lint, here we just take the first.
            // If there are no suggestions, then apply just collect the message.
            let mut new_text = text.chars().collect_vec();
            let mut messages = Vec::new();
            for lint in lints {
                if (matches!(lint.lint_kind, LintKind::Formatting) || options.should_fix)
                    && let Some(first) = lint.suggestions.first()
                {
                    first.apply(lint.span, &mut new_text);
                } else {
                    messages.push(lint_to_compilation_message(lint));
                }
            }
            let new_text = new_text.iter().collect();

            (Some(new_text), messages)
        } else {
            let messages = lints
                .into_iter()
                .map(lint_to_compilation_message)
                .collect_vec();
            (None, messages)
        };
        let messages = (!messages.is_empty()).then_some(messages);

        let authors = Some(vec![
            SoftwareApplication::new("Harper".to_string()).into_author_role(
                AuthorRoleName::Linter,
                Some(Format::Text),
                Some(Timestamp::now()),
            ),
        ]);

        Ok(LintingOutput {
            messages,
            authors,
            code: new_text,
            ..Default::default()
        })
    }
}

/// Convert a Harper [Lint] to a Stencila [CompilationMessage]
fn lint_to_compilation_message(lint: Lint) -> CompilationMessage {
    use LintKind::*;
    let level = match lint.lint_kind {
        BoundaryError | Capitalization | Eggcorn | Malapropism | Spelling | Typo => {
            MessageLevel::Error
        }
        Formatting | Grammar | Punctuation | Readability | Redundancy | Regionalism
        | Repetition => MessageLevel::Warning,
        Style | Usage | WordChoice | _ => MessageLevel::Info,
    };

    let lint_kind = lint.lint_kind.to_string();

    let mut message = lint.message;
    // Extend message for certain kinds of lints (for many lint kinds, the message already contains the suggestion)
    if !lint.suggestions.is_empty() && matches!(lint.lint_kind, LintKind::Spelling) {
        message.push_str(" Suggestion");
        if lint.suggestions.len() > 1 {
            message.push('s');
        }
        message.push_str(": ");

        for (index, suggestion) in lint.suggestions.iter().enumerate() {
            if index > 0 {
                message.push_str(", ");
            }
            message.push_str(&match suggestion {
                Suggestion::ReplaceWith(chars) => ["`", &String::from_iter(chars), "`"].concat(),
                Suggestion::InsertAfter(chars) => {
                    ["add `", &String::from_iter(chars), "`"].concat()
                }
                Suggestion::Remove => "remove".to_string(),
            });
        }
        message.push('.');
    }

    let location = CodeLocation {
        start_line: Some(0),
        start_column: Some(lint.span.start as u64),
        end_column: Some(lint.span.end as u64),
        ..Default::default()
    };

    CompilationMessage {
        level,
        error_type: Some(lint_kind),
        message,
        code_location: Some(location),
        ..Default::default()
    }
}
