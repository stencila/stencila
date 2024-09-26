use std::path::PathBuf;

use codecs::DecodeOptions;
use schema::{Article, Block, IncludeBlock};

use crate::{interrupt_impl, prelude::*, Phase};

impl Executable for IncludeBlock {
    #[tracing::instrument(skip_all)]
    async fn compile(&mut self, executor: &mut Executor) -> WalkControl {
        // Return early if no source, has content or execution status
        if self.source.trim().is_empty()
            || self.content.is_some()
            || self.options.execution_status.is_some()
        {
            return WalkControl::Continue;
        }

        let node_id = self.node_id();
        tracing::trace!("Compiling IncludeBlock {node_id}");

        // Note that this is a simplified version of what happens in execute().
        // In particular, there is NO recursive execution of the content.

        let started = Timestamp::now();

        // Get the content from the source
        let (content, pop_dir, messages) =
            source_to_content(&self.source, &self.media_type, executor).await;

        // Patch the content on the include
        let op = if let Some(content) = content {
            set(NodeProperty::Content, content)
        } else {
            none(NodeProperty::Content)
        };
        executor.patch(&node_id, [op]);

        // Pop off the directory stack if necessary
        if pop_dir {
            executor.directory_stack.pop();
        }

        let messages = (!messages.is_empty()).then_some(messages);

        let ended = Timestamp::now();

        let status = execution_status(&messages);
        let required = execution_required_status(&status);
        let duration = execution_duration(&started, &ended);
        let count = self.options.execution_count.unwrap_or_default() + 1;

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

        // Continue walk to compile nodes in the content
        WalkControl::Continue
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
            tracing::trace!("Skipping IncludeBlock {node_id}");
            return WalkControl::Break;
        }

        tracing::debug!("Executing IncludeBlock {node_id}");

        executor.patch(
            &node_id,
            [
                set(NodeProperty::ExecutionStatus, ExecutionStatus::Running),
                none(NodeProperty::ExecutionMessages),
            ],
        );

        // Include the source (if it is not empty)
        let source = self.source.trim();
        if !source.is_empty() {
            let started = Timestamp::now();

            // Get the content for the source
            let (mut content, pop_dir, mut messages) =
                source_to_content(source, &self.media_type, executor).await;

            if let Some(content) = &mut content {
                // Clear any existing content while ensuring an array to append to
                let reset = if self.content.is_some() {
                    clear(NodeProperty::Content)
                } else {
                    set(NodeProperty::Content, Vec::<Block>::new())
                };

                // Append the content as a Vec<Block> to avoid loosing ids which
                // may be needed when executing the content (which would happen if used set)
                executor.patch(
                    &node_id,
                    [reset, append(NodeProperty::Content, content.clone())],
                );

                // Execute the content
                if let Err(error) = content.walk_async(executor).await {
                    messages.push(error_to_execution_message("While executing content", error));
                }
            } else {
                executor.patch(&node_id, [none(NodeProperty::Content)]);
            }

            // Pop off the directory stack if necessary
            if pop_dir {
                executor.directory_stack.pop();
            }

            let messages = (!messages.is_empty()).then_some(messages);

            let ended = Timestamp::now();

            let status = execution_status(&messages);
            let required = execution_required_status(&status);
            let duration = execution_duration(&started, &ended);
            let count = self.options.execution_count.unwrap_or_default() + 1;

            if matches!(executor.phase, Phase::ExecuteWithoutPatches) {
                self.content = content;
                self.options.execution_messages = messages.clone();
            } else {
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
            }
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
) -> (Option<Vec<Block>>, bool, Vec<ExecutionMessage>) {
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
                    messages.push(ExecutionMessage::new(
                        MessageLevel::Error,
                        "Expected source to be an article, got `{node}`".to_string(),
                    ));
                    None
                }
            }
        }
        Err(error) => {
            messages.push(error_to_execution_message("While decoding source", error));
            None
        }
    };

    // TODO: Implement sub-selecting from included based on `select`

    (content, pop_dir, messages)
}
