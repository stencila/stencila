use std::path::PathBuf;

use stencila_codec_biblio::decode;
use stencila_codecs::Format;
use stencila_schema::{Bibliography, CompilationMessage, Reference};

use crate::prelude::*;

impl Executable for Bibliography {
    #[tracing::instrument(skip_all)]
    async fn compile(&mut self, executor: &mut Executor) -> WalkControl {
        // Return early if no source
        if self.source.trim().is_empty() {
            return WalkControl::Continue;
        }

        let node_id = self.node_id();
        tracing::trace!("Compiling Bibliography {node_id}");

        // Get the references from the source
        let (references, messages) =
            source_to_references(&self.source, &self.media_type, executor).await;

        // Update the bibliography's references
        if let Some(references) = references {
            self.references = Some(references.clone());
            executor.patch(
                &node_id,
                [
                    none(NodeProperty::References),
                    append(NodeProperty::References, references),
                ],
            );
        } else {
            self.references = None;
            executor.patch(&node_id, [none(NodeProperty::References)]);
        }

        let messages = (!messages.is_empty()).then_some(messages);
        self.options.compilation_messages = messages.clone();
        executor.patch(&node_id, [set(NodeProperty::CompilationMessages, messages)]);

        WalkControl::Break
    }
}

/// Get references from a bibliography source file
async fn source_to_references(
    source: &str,
    media_type: &Option<String>,
    executor: &mut Executor,
) -> (Option<Vec<Reference>>, Vec<CompilationMessage>) {
    let mut messages = Vec::new();

    // Make the path relative to the last directory in the executor's directory stack
    let last_dir = executor.directory_stack.last();
    let path = last_dir
        .map(|dir| dir.join(source))
        .unwrap_or_else(|| PathBuf::from(source));

    // Read the file content
    let content = match tokio::fs::read_to_string(&path).await {
        Ok(content) => content,
        Err(error) => {
            messages.push(CompilationMessage::new(
                MessageLevel::Error,
                format!(
                    "Unable to read bibliography file '{}': {error}",
                    path.display()
                ),
            ));
            return (None, messages);
        }
    };

    // Determine the format from media_type or path
    let format = media_type
        .as_ref()
        .and_then(|mt| Format::from_media_type(mt).ok())
        .unwrap_or_else(|| Format::from_path(&path));

    // Decode the content based on format
    let references = match format {
        Format::Yaml => match decode::yaml(&content) {
            Ok(refs) => Some(refs),
            Err(error) => {
                messages.push(CompilationMessage::new(
                    MessageLevel::Error,
                    format!("Unable to parse YAML bibliography: {error}"),
                ));
                None
            }
        },
        Format::Bibtex => match decode::bibtex(&content) {
            Ok(refs) => Some(refs),
            Err(error) => {
                messages.push(CompilationMessage::new(
                    MessageLevel::Error,
                    format!("Unable to parse BibTeX bibliography: {error}"),
                ));
                None
            }
        },
        _ => {
            // Fallback to text parsing
            let refs = decode::text_to_references(&content);
            if refs.is_empty() { None } else { Some(refs) }
        }
    };

    (references, messages)
}
