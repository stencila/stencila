use std::{
    env,
    fs::{read_to_string, write},
    path::Path,
};

use stencila_linter::{
    Format, Linter, LinterAvailability, LintingOptions, LintingOutput, NodeType,
    common::{
        async_trait::async_trait,
        eyre::{OptionExt, Result},
        serde::Deserialize,
        serde_json,
        tempfile::NamedTempFile,
        tracing,
    },
    schema::{
        AuthorRoleName, CodeLocation, CompilationMessage, MessageLevel, SoftwareApplication,
        Timestamp,
    },
};
use tools::{Pyright, Tool};

#[derive(Default)]
pub struct PyrightLinter;

#[async_trait]
impl Linter for PyrightLinter {
    fn name(&self) -> &str {
        "pyright"
    }

    fn node_types(&self) -> Vec<NodeType> {
        vec![NodeType::CodeChunk, NodeType::CodeExpression]
    }

    fn formats(&self) -> Vec<Format> {
        vec![Format::Python]
    }

    fn availability(&self) -> LinterAvailability {
        if Pyright.is_installed() {
            LinterAvailability::Available
        } else if Pyright.is_installable() {
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
        tracing::trace!("Linting with Pyright");

        // Resolve directory for detection of the environment when running Pyright
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

        // Run Pyright with JSON output to parse into messages
        // See https://github.com/Microsoft/pyright/blob/main/docs/command-line.md
        let mut pyright = Pyright.command();
        pyright.current_dir(dir).arg("--outputjson");

        // Search up the tree from the document for Python virtual environment
        // so that correct dependencies are available and spurious
        // MissingImports errors are avoided.
        let mut dir = dir.to_path_buf();
        let mut found_venv = false;
        loop {
            let python_path = dir.join(".venv").join("bin").join("python");
            if python_path.exists() {
                pyright.arg(format!("--pythonpath={}", python_path.display()));
                found_venv = true;
                break;
            }

            if !dir.pop() {
                break;
            }
        }

        // Fallback: use PYTHON_PATH environment variable if no venv found
        if !found_venv && let Ok(python_path) = env::var("PYTHON_PATH") {
            pyright.arg(format!("--pythonpath={python_path}"));
        }

        if let Ok(output) = pyright.arg(temp_path).output() {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let diagnostics = serde_json::from_str::<PyrightDiagnostics>(&stdout)?;

            // Successfully ran Pyright so add as an author (regardless of whether it made any fixes)
            authors.push(
                SoftwareApplication::new("Pyright".to_string()).into_author_role(
                    AuthorRoleName::Linter,
                    Some(Format::Python),
                    Some(Timestamp::now()),
                ),
            );

            // Convert each Pyright diagnostic to a compilation message
            for diag in diagnostics.general_diagnostics {
                // Ignore some diagnostics which do not make so much sense in code cells
                if matches!(diag.rule.as_deref(), Some("reportUnusedExpression")) {
                    continue;
                }

                messages.push(CompilationMessage::from(diag));
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
            code,
        })
    }
}

// A diagnostic report from Pyright
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PyrightDiagnostics {
    general_diagnostics: Vec<PyrightDiagnostic>,
}
#[derive(Deserialize)]
struct PyrightDiagnostic {
    rule: Option<String>,
    severity: String,
    message: String,
    range: PyrightRange,
}
#[derive(Deserialize)]
struct PyrightRange {
    start: PyrightLocation,
    end: PyrightLocation,
}
#[derive(Deserialize)]
struct PyrightLocation {
    line: u64,
    character: u64,
}

impl From<PyrightDiagnostic> for CompilationMessage {
    fn from(diag: PyrightDiagnostic) -> Self {
        let code_location = Some(CodeLocation {
            start_line: Some(diag.range.start.line),
            start_column: Some(diag.range.start.character),
            end_line: Some(diag.range.end.line),
            end_column: Some(diag.range.end.character),
            ..Default::default()
        });

        let level = match diag.severity.as_str() {
            "warning" => MessageLevel::Warning,
            _ => MessageLevel::Error,
        };

        let message = format!(
            "{}{}",
            diag.message,
            diag.rule
                .map(|rule| format!(" (Pyright {})", rule.trim_start_matches("report")))
                .unwrap_or_default()
        )
        .trim()
        .to_string();

        Self {
            error_type: Some(format!("Linting {}", level.to_string().to_lowercase())),
            level,
            message,
            code_location,
            ..Default::default()
        }
    }
}
