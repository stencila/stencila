use std::path::PathBuf;

use stencila_codecs::DecodeOptions;
use stencila_schema::{Block, CompilationMessage, IncludeBlock};

use crate::prelude::*;

impl Executable for IncludeBlock {
    #[tracing::instrument(skip_all)]
    async fn compile(&mut self, executor: &mut Executor) -> WalkControl {
        // Return early if no source
        // TODO: should also return early if source has not changed since last compile
        if self.source.trim().is_empty() {
            // Continue walk to compile any existing `content`
            return WalkControl::Continue;
        }

        let node_id = self.node_id();
        tracing::trace!("Compiling IncludeBlock {node_id}");

        // Get the content from the source
        let (content, pop_dir, mut messages) =
            source_to_content(&self.source, &self.media_type, executor).await;

        // Add the content to the include block
        if let Some(content) = content {
            self.content = Some(content.clone());
            executor.patch(
                &node_id,
                [
                    // It is important to use `none` and `append` here because
                    // the later retains node ids so they are the same as in `self.content`
                    none(NodeProperty::Content),
                    append(NodeProperty::Content, content),
                ],
            );
        } else {
            self.content = None;
            executor.patch(&node_id, [none(NodeProperty::Content)])
        };

        // Compile the content. This needs to be done here between (possibly)
        // pushing and popping from the directory stack.
        if let Err(error) = self.content.walk_async(executor).await {
            messages.push(error_to_compilation_message(error));
        };

        // Pop off the directory stack if necessary
        if pop_dir {
            executor.directory_stack.pop();
        }

        let messages = (!messages.is_empty()).then_some(messages);

        self.options.compilation_messages = messages.clone();
        executor.patch(&node_id, [set(NodeProperty::CompilationMessages, messages)]);

        // Break because `content` already compiled above
        WalkControl::Break
    }

    #[tracing::instrument(skip_all)]
    async fn prepare(&mut self, _executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::trace!("Preparing IncludeBlock {node_id}");

        // Continue walk to prepare nodes in `content`
        WalkControl::Continue
    }

    #[tracing::instrument(skip_all)]
    async fn execute(&mut self, _executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::debug!("Executing IncludeBlock {node_id}: {}", self.source);

        // Continue walk to execute nodes in `content`
        WalkControl::Continue
    }

    #[tracing::instrument(skip_all)]
    async fn interrupt(&mut self, _executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::debug!("Interrupting IncludeBlock {node_id}");

        // Continue walk to interrupt nodes in `content`
        WalkControl::Continue
    }
}

// Get the content from a source
async fn source_to_content(
    source: &str,
    media_type: &Option<String>,
    executor: &mut Executor,
) -> (Option<Vec<Block>>, bool, Vec<CompilationMessage>) {
    let mut messages = Vec::new();

    // Resolve the source into a fully qualified URL (including `file://` URL)
    let (identifier, pop_dir) = if source.starts_with("https://") || source.starts_with("http://") {
        (source.to_string(), false)
    } else {
        // Make the path relative to the last directory in the executor's directory stack
        // and update the stack if necessary.
        let last_dir = executor.directory_stack.last();
        let path = last_dir
            .map(|dir| dir.join(source))
            .unwrap_or_else(|| PathBuf::from(source));
        let pop_dir = if let Some(dir) = path.parent() {
            if Some(dir) != last_dir.map(|path_buf| path_buf.as_ref()) {
                executor.directory_stack.push(dir.to_path_buf());
                true
            } else {
                false
            }
        } else {
            false
        };

        (path.to_string_lossy().to_string(), pop_dir)
    };

    // Decode the identifier
    let content: Option<Vec<Block>> = match stencila_codecs::from_identifier(
        &identifier,
        Some(DecodeOptions {
            media_type: media_type.clone(),
            // Set format to None so that the format of the executor's decode options
            // (that of the executor's document) is not used when decoding
            format: None,
            ..executor.decode_options.clone().unwrap_or_default()
        }),
    )
    .await
    {
        Ok(node) => {
            // Transform the decoded node into a blocks
            match node.try_into() {
                Ok(blocks) => Some(blocks),
                Err(error) => {
                    messages.push(CompilationMessage::new(
                        MessageLevel::Error,
                        format!("Unable to convert source into block content: {error}"),
                    ));
                    None
                }
            }
        }
        Err(error) => {
            messages.push(error_to_compilation_message(error));
            None
        }
    };

    // TODO: Implement sub-selecting from included based on `select`

    (content, pop_dir, messages)
}
