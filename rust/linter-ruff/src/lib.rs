use std::{
    fs::{read_to_string, write},
    path::Path,
};

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
use tools::{Ruff, Tool};

#[derive(Default)]
pub struct RuffLinter;

#[async_trait]
impl Linter for RuffLinter {
    fn name(&self) -> &str {
        "ruff"
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
        vec![Format::Python]
    }

    fn supports_formatting(&self) -> bool {
        true
    }

    fn supports_fixing(&self) -> bool {
        true
    }

    fn availability(&self) -> LinterAvailability {
        if Ruff.is_installed() {
            LinterAvailability::Available
        } else if Ruff.is_installable() {
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
        tracing::trace!("Linting with Ruff");

        // Resolve directory for detection of the environment when running Ruff
        let dir = if path.is_file() {
            path.parent().ok_or_eyre("unable to resolve directory")?
        } else {
            path
        };

        // Write the code to a temporary file. Avoid temptation to add any import
        // before the code as that mucks up line numbers using for matching
        let temp_file = NamedTempFile::new()?;
        let temp_path = temp_file.path();
        write(temp_path, code)?;

        let mut messages = Vec::new();
        let mut authors = Vec::new();

        // Format code if specified
        if options.should_format {
            let result = Ruff
                .command()
                .current_dir(dir)
                .arg("format")
                .arg(temp_path)
                .output();

            if let Ok(output) = result {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                if stdout.contains("reformatted") {
                    // Successfully ran Ruff, and it made changes, so add as an author
                    authors.push(
                        SoftwareApplication::new("Ruff".to_string()).into_author_role(
                            AuthorRoleName::Formatter,
                            Some(Format::Python),
                            Some(Timestamp::now()),
                        ),
                    );
                }
            }
        }

        // Run Ruff with JSON output for parsing of diagnostic to messages
        let mut ruff = Ruff.command();
        ruff.current_dir(dir)
            .args(["check", "--output-format=json"])
            .arg(temp_path);
        if options.should_fix {
            ruff.arg("--fix");
        }
        if let Ok(output) = ruff.output() {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let ruff_messages = serde_json::from_str::<Vec<RuffMessage>>(&stdout)?;

            // Successfully ran Ruff so add as an author (regardless of whether it made any fixes)
            authors.push(
                SoftwareApplication::new("Ruff".to_string()).into_author_role(
                    AuthorRoleName::Linter,
                    Some(Format::Python),
                    Some(Timestamp::now()),
                ),
            );

            // Convert each Ruff message to a compilation message
            for ruff_message in ruff_messages {
                // Ignore some messages which make no sense when concatenating code chunks
                // E402: Module level import not at top of file
                if matches!(ruff_message.code.as_deref(), Some("E402")) {
                    continue;
                }

                messages.push(CompilationMessage::from(ruff_message));
            }
        }

        // Read the updated file if formatted or fixed
        let code = if options.should_format || options.should_fix {
            let new_code = read_to_string(temp_path)?;
            (new_code != code).then_some(new_code)
        } else {
            None
        };

        Ok(LintingOutput {
            authors: (!authors.is_empty()).then_some(authors),
            messages: (!messages.is_empty()).then_some(messages),
            content: code,
        })
    }
}

// A diagnostic message from Ruff
#[derive(Deserialize)]
struct RuffMessage {
    code: Option<String>,
    message: String,
    location: Option<RuffLocation>,
    end_location: Option<RuffLocation>,
}

#[derive(Deserialize)]
struct RuffLocation {
    column: u64,
    row: u64,
}

impl From<RuffMessage> for CompilationMessage {
    fn from(message: RuffMessage) -> Self {
        Self {
            error_type: Some("Linting warning".into()),
            level: MessageLevel::Warning,
            message: format!(
                "{}{}",
                message.message,
                message
                    .code
                    .map(|code| format!(" (Ruff {code})"))
                    .unwrap_or_default()
            ),
            code_location: Some(CodeLocation {
                // Note that Ruff provides 1-based row and column indices
                start_line: message
                    .location
                    .as_ref()
                    .map(|location| location.row.saturating_sub(1)),
                start_column: message
                    .location
                    .as_ref()
                    .map(|location| location.column.saturating_sub(1)),
                end_line: message
                    .end_location
                    .as_ref()
                    .map(|location| location.row.saturating_sub(1)),
                end_column: message
                    .end_location
                    .as_ref()
                    .map(|location| location.column.saturating_sub(1)),
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}
