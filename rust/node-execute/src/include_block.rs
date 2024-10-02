use std::path::PathBuf;

use codecs::DecodeOptions;
use schema::{Article, Block, CompilationMessage, IncludeBlock};

use crate::{interrupt_impl, prelude::*};

impl Executable for IncludeBlock {
    #[tracing::instrument(skip_all)]
    async fn compile(&mut self, executor: &mut Executor) -> WalkControl {
        // Return early if no source, or already has content
        if self.source.trim().is_empty() || self.content.is_some() {
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
    async fn prepare(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::trace!("Preparing IncludeBlock {node_id}");

        // Set execution status
        if let Some(status) = executor.node_execution_status(
            self.node_type(),
            &node_id,
            &self.execution_mode,
            &self.options.compilation_digest,
            &self.options.execution_digest,
        ) {
            self.options.execution_status = Some(status.clone());
            executor.patch(&node_id, [set(NodeProperty::ExecutionStatus, status)]);
        }

        // Continue to mark executable nodes in `content` as pending
        WalkControl::Continue
    }

    #[tracing::instrument(skip_all)]
    async fn execute(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();

        if !matches!(
            self.options.execution_status,
            Some(ExecutionStatus::Pending)
        ) {
            tracing::trace!("Skipping IncludeBlock {node_id}: {}", self.source);
            return WalkControl::Break;
        }

        tracing::debug!("Executing IncludeBlock {node_id}: {}", self.source);

        executor.patch(
            &node_id,
            [
                set(NodeProperty::ExecutionStatus, ExecutionStatus::Running),
                none(NodeProperty::ExecutionMessages),
            ],
        );

        // Include the source (if it is not empty)
        if self.content.is_some() {
            let mut messages = Vec::new();
            let started = Timestamp::now();

            // Execute the content
            if let Err(error) = self.content.walk_async(executor).await {
                messages.push(error_to_execution_message("While executing content", error));
            }

            let messages = (!messages.is_empty()).then_some(messages);

            let ended = Timestamp::now();

            let status = execution_status(&messages);
            let required = execution_required_status(&status);
            let duration = execution_duration(&started, &ended);
            let count = self.options.execution_count.unwrap_or_default() + 1;

            // Set properties that may be using in rendering
            self.options.execution_messages = messages.clone();

            executor.patch(
                &node_id,
                [
                    set(NodeProperty::ExecutionStatus, status),
                    set(NodeProperty::ExecutionRequired, required),
                    set(NodeProperty::ExecutionMessages, messages),
                    set(NodeProperty::ExecutionDuration, duration),
                    set(NodeProperty::ExecutionEnded, ended),
                    set(NodeProperty::ExecutionCount, count),
                ],
            );
        } else {
            executor.patch(
                &node_id,
                [
                    set(NodeProperty::ExecutionStatus, ExecutionStatus::Empty),
                    set(NodeProperty::ExecutionRequired, ExecutionRequired::No),
                    none(NodeProperty::ExecutionDuration),
                    none(NodeProperty::ExecutionEnded),
                ],
            );
        }

        // Break walk because already executed `content`
        WalkControl::Break
    }

    #[tracing::instrument(skip_all)]
    async fn interrupt(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::debug!("Interrupting IncludeBlock {node_id}");

        interrupt_impl!(self, executor, &node_id);

        // Continue to interrupt executable nodes in `content`
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
    let (url, pop_dir) = if source.starts_with("https://") || source.starts_with("http://") {
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

        (["file://", &path.to_string_lossy()].concat(), pop_dir)
    };

    // Decode the URL
    let content: Option<Vec<Block>> = match codecs::from_url(
        &url,
        Some(DecodeOptions {
            media_type: media_type.clone(),
            ..Default::default()
        }),
    )
    .await
    {
        Ok(node) => {
            // Transform the decoded node into a blocks
            match node {
                Node::Article(Article { content, .. }) => Some(content),
                _ => {
                    messages.push(CompilationMessage::new(
                        MessageLevel::Error,
                        "Expected source to be an article, got `{node}`".to_string(),
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
