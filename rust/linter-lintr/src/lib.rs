use std::{fs::write, path::Path};

use serde::Deserialize;
use tempfile::NamedTempFile;

use stencila_linter::{
    Format, Linter, LinterAvailability, LintingOptions, LintingOutput, NodeType, async_trait,
    eyre::{OptionExt, Result},
    schema::{
        AuthorRoleName, CodeLocation, CompilationMessage, MessageLevel, SoftwareApplication,
        Timestamp,
    },
};
use tools::{LintR, R, Tool};

#[derive(Default)]
pub struct LintRLinter;

#[async_trait]
impl Linter for LintRLinter {
    fn name(&self) -> &str {
        "lintr"
    }

    fn node_types(&self) -> Vec<NodeType> {
        vec![
            NodeType::CodeChunk,
            NodeType::CodeExpression,
            NodeType::CodeBlock,
            NodeType::CodeInline,
        ]
    }

    fn formats(&self) -> Vec<Format> {
        vec![Format::R]
    }

    fn supports_formatting(&self) -> bool {
        false
    }

    fn supports_fixing(&self) -> bool {
        false
    }

    fn availability(&self) -> LinterAvailability {
        if LintR.is_installed() {
            LinterAvailability::Available
        } else if LintR.is_installable() {
            LinterAvailability::Installable
        } else {
            LinterAvailability::Unavailable
        }
    }

    #[tracing::instrument(skip(self, code))]
    async fn lint(
        &self,
        code: &str,
        path: &Path,
        options: &LintingOptions,
    ) -> Result<LintingOutput> {
        tracing::trace!("Linting with LintR");

        // Resolve directory for detection of the environment when running LintR
        let dir = if path.is_file() {
            path.parent().ok_or_eyre("unable to resolve directory")?
        } else {
            path
        };

        // Write the code to a temporary file. Avoid temptation to add any import
        // before the code as that mucks up line numbers using for matching
        let temp_file = NamedTempFile::new()?;
        let temp_path = temp_file.path();
        let temp_path_str = temp_path.to_string_lossy();
        write(temp_path, code)?;

        let mut messages = Vec::new();
        let mut authors = Vec::new();

        // Run LintR with JSON output for parsing of diagnostic to messages
        let mut r = R.command();
        r.current_dir(dir).arg("-e").arg(format!(
            "jsonlite::toJSON(lintr::lint('{temp_path_str}'), auto_unbox=T)"
        ));
        if let Ok(output) = r.output() {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let ruff_messages = serde_json::from_str::<Vec<LintRMessage>>(&stdout)?;

            // Successfully ran LintR so add as an author (regardless of whether it made any fixes)
            authors.push(
                SoftwareApplication::new("LintR".to_string()).into_author_role(
                    AuthorRoleName::Linter,
                    Some(Format::Python),
                    Some(Timestamp::now()),
                ),
            );

            // Convert each LintR message to a compilation message
            for ruff_message in ruff_messages {
                messages.push(CompilationMessage::from(ruff_message));
            }
        }

        Ok(LintingOutput {
            authors: (!authors.is_empty()).then_some(authors),
            messages: (!messages.is_empty()).then_some(messages),
            ..Default::default()
        })
    }
}

// A diagnostic message from lintr
#[derive(Deserialize)]
struct LintRMessage {
    r#type: String,
    message: String,
    line_number: u64,
    column_number: u64,
}

impl From<LintRMessage> for CompilationMessage {
    fn from(message: LintRMessage) -> Self {
        let error_type = {
            Some(format!(
                "Linting {}",
                match message.r#type.as_str() {
                    "style" => "advice",
                    typ => typ,
                }
            ))
        };
        let level = match message.r#type.as_str() {
            "style" => MessageLevel::Debug,
            "warning" => MessageLevel::Warning,
            _ => MessageLevel::Error,
        };

        Self {
            error_type,
            level,
            message: message.message,
            code_location: Some(CodeLocation {
                start_line: Some(message.line_number.saturating_sub(1)),
                start_column: Some(message.column_number.saturating_sub(1)),
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}
