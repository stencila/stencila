use std::{
    fs::{read_to_string, write},
    path::Path,
};

use stencila_linter::{
    Format, Linter, LinterAvailability, LintingOptions, LintingOutput, NodeType, async_trait,
    eyre::{OptionExt, Result},
    stencila_schema::{AuthorRoleName, SoftwareApplication, Timestamp},
};
use stencila_tools::{R, StyleR, Tool};

#[derive(Default)]
pub struct StyleRLinter;

#[async_trait]
impl Linter for StyleRLinter {
    fn name(&self) -> &str {
        "styler"
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
        true
    }

    fn supports_fixing(&self) -> bool {
        false
    }

    fn availability(&self) -> LinterAvailability {
        if StyleR.is_installed() {
            LinterAvailability::Available
        } else if StyleR.is_installable() {
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
        // Return early if not formattin
        if !options.should_format {
            return Ok(LintingOutput::default());
        }

        tracing::trace!("Linting with Styler");

        // Resolve directory for detection of the environment when running StyleR
        let dir = if path.is_file() {
            path.parent().ok_or_eyre("unable to resolve directory")?
        } else {
            path
        };

        // Write the code to a temporary file. Avoid temptation to add any import
        // before the code as that mucks up line numbers using for matching
        let temp_file = tempfile::Builder::new().suffix(".R").tempfile()?;
        let temp_path = temp_file.path();
        let temp_path_str = temp_path.to_string_lossy();
        write(temp_path, code)?;

        // Run styler and read updated code
        R.command()
            .current_dir(dir)
            .arg("-e")
            .arg(format!(
                "styler::style_file('{temp_path_str}', strict=TRUE)"
            ))
            .output()?;
        let new_code = read_to_string(temp_path)?;

        // Return with no output if no change in code
        if new_code == code {
            return Ok(LintingOutput::default());
        }

        Ok(LintingOutput {
            content: Some(new_code),
            authors: Some(vec![
                SoftwareApplication::new("Styler".to_string()).into_author_role(
                    AuthorRoleName::Formatter,
                    Some(Format::R),
                    Some(Timestamp::now()),
                ),
            ]),
            ..Default::default()
        })
    }
}
