use std::{
    env::current_dir,
    path::{Path, PathBuf},
};

pub use stencila_linter::LintingOptions;
use stencila_linter::{
    Linter, LinterAvailability, LintingOutput,
    common::{
        eyre::{Result, bail},
        itertools::Itertools,
        tracing,
    },
};
use stencila_linter_harper::HarperLinter;
use stencila_linter_lintr::LintRLinter;
use stencila_linter_pyright::PyrightLinter;
use stencila_linter_ruff::RuffLinter;
use stencila_linter_styler::StyleRLinter;

pub mod cli;

/// Get a list of available linters
pub async fn list() -> Vec<Box<dyn Linter>> {
    vec![
        // The order here is important it is used in places like
        // `stencila linters list` and other user interfaces

        // Programming languages
        Box::<RuffLinter>::default() as Box<dyn Linter>,
        Box::<PyrightLinter>::default() as Box<dyn Linter>,
        Box::<LintRLinter>::default() as Box<dyn Linter>,
        Box::<StyleRLinter>::default() as Box<dyn Linter>,

        // Grammar and spelling
        Box::<HarperLinter>::default() as Box<dyn Linter>,
    ]
}

/// Lint some content
#[tracing::instrument(skip(content))]
pub async fn lint(
    content: &str,
    path: Option<&Path>,
    options: LintingOptions,
) -> Result<Vec<LintingOutput>> {
    tracing::trace!("Linting");

    // For proper detection of the environment the path should not be empty.
    // Canonicalization also helps for paths that are providing to some linting
    // tools
    let path = match path {
        Some(path) => {
            if path == PathBuf::new() {
                current_dir()?
            } else {
                path.to_path_buf()
            }
        }
        None => current_dir()?,
    };

    // Filter the list of linters for those that are available and will handle the content
    let linters = list()
        .await
        .into_iter()
        .filter(|linter| {
            if let Some(name) = &options.linter
                && linter.name() != name.to_lowercase()
            {
                return false;
            }

            if let Some(format) = &options.format
                && !linter.formats().contains(format)
            {
                return false;
            }

            if let Some(node_type) = &options.node_type
                && !linter.node_types().contains(node_type)
            {
                return false;
            }

            matches!(linter.availability(), LinterAvailability::Available)
        })
        .collect_vec();

    // Exit early if no matching linters found
    if linters.is_empty() {
        if let Some(name) = &options.linter {
            bail!("No linters with name matching `{name}`");
        }

        let mut message = "No linters available".to_string();

        if let Some(format) = &options.format {
            message.push_str(" for format `");
            message.push_str(format.name());
            message.push('`');
        }

        if let Some(node_type) = &options.node_type {
            message.push_str(" for node type `");
            message.push_str(&node_type.to_string());
            message.push('`');
        }

        bail!(message)
    }

    // Perform linting with each matching linter
    let mut outputs = Vec::new();
    for linter in linters {
        let output = match linter.lint(content, &path, &options).await {
            Ok(output) => output,
            Err(error) => {
                tracing::warn!("Linter `{}` failed: {error}", linter.name());
                continue;
            }
        };
        outputs.push(output);
    }

    Ok(outputs)
}
