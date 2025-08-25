use std::{
    collections::HashMap,
    env::current_dir,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

pub use stencila_linter::LintingOptions;
use stencila_linter::{
    Linter, LinterAvailability, LintingOutput,
    common::{
        eyre::{Result, bail},
        itertools::Itertools,
        once_cell::sync::Lazy,
        tracing,
    },
};
use stencila_linter_harper::HarperLinter;
use stencila_linter_links::LinksLinter;
use stencila_linter_lintr::LintRLinter;
use stencila_linter_pyright::PyrightLinter;
use stencila_linter_ruff::RuffLinter;
use stencila_linter_styler::StyleRLinter;

pub mod cli;

/// Get a list of available linters
pub async fn list() -> Vec<Box<dyn Linter>> {
    vec![
        // The order here is important it is used in places like `stencila
        // linters list` and other user interfaces. Generally place linters that
        // make formatting changes to the code before those that are only
        // checkers.

        // Programming languages
        Box::<RuffLinter>::default() as Box<dyn Linter>,
        Box::<PyrightLinter>::default() as Box<dyn Linter>,
        Box::<StyleRLinter>::default() as Box<dyn Linter>,
        Box::<LintRLinter>::default() as Box<dyn Linter>,
        // Grammar and spelling
        Box::<HarperLinter>::default() as Box<dyn Linter>,
        // Content validation
        Box::<LinksLinter>::default() as Box<dyn Linter>,
    ]
}

/// Lint some content
#[tracing::instrument(skip(content))]
pub async fn lint(
    content: &str,
    path: Option<&Path>,
    options: LintingOptions,
) -> Result<LintingOutput> {
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

    // Cache linter availability to avoid relatively expensive
    // calls to `linter.availability()` on each call of this function.
    static LINTER_AVAILABLE: Lazy<Arc<Mutex<HashMap<String, bool>>>> = Lazy::new(Arc::default);

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

            // Check availability using cache with fallback (in case of unlikely mutex poisoning)
            match LINTER_AVAILABLE.lock() {
                Ok(mut guard) => {
                    *(guard.entry(linter.name().to_string()).or_insert_with(|| {
                        matches!(linter.availability(), LinterAvailability::Available)
                    }))
                }
                Err(error) => {
                    tracing::warn!("Unable to lock linter availability cache: {error}");
                    matches!(linter.availability(), LinterAvailability::Available)
                }
            }
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

    // Perform linting with each matching linter. If there is any output from linting
    // (due to formatting or fixing) then pass on to the next linter
    let mut current_content = content.to_string();
    let mut accumulated_output = LintingOutput::default();

    for linter in linters {
        let output = match linter.lint(&current_content, &path, &options).await {
            Ok(output) => output,
            Err(error) => {
                tracing::warn!("Linter `{}` failed: {error}", linter.name());
                continue;
            }
        };

        // Accumulate messages
        if let Some(messages) = output.messages {
            accumulated_output.messages = match accumulated_output.messages {
                Some(mut existing) => {
                    existing.extend(messages);
                    Some(existing)
                }
                None => Some(messages),
            };
        }

        // Accumulate authors
        if let Some(authors) = output.authors {
            accumulated_output.authors = match accumulated_output.authors {
                Some(mut existing) => {
                    existing.extend(authors);
                    Some(existing)
                }
                None => Some(authors),
            };
        }

        // Update current content for next linter if this linter changed it
        if let Some(new_content) = output.content {
            current_content = new_content;
        }
    }

    // Set the final content if it changed from the original
    if current_content != content {
        accumulated_output.content = Some(current_content);
    }

    Ok(accumulated_output)
}
