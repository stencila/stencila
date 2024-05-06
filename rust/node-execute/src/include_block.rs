use codecs::DecodeOptions;
use schema::{Article, Block, IncludeBlock};

use crate::{interrupt_impl, pending_impl, prelude::*};

impl Executable for IncludeBlock {
    #[tracing::instrument(skip_all)]
    async fn pending(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::debug!("Pending IncludeBlock {node_id}");

        pending_impl!(executor, &node_id);

        // Continue to mark executable nodes in `content` as pending
        WalkControl::Continue
    }

    #[tracing::instrument(skip_all)]
    async fn execute(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();

        if !executor.should_execute_code(
            &node_id,
            &self.auto_exec,
            &self.options.compilation_digest,
            &self.options.execution_digest,
        ) {
            tracing::debug!("Skipping IncludeBlock {node_id}");

            return WalkControl::Break;
        }

        tracing::trace!("Executing IncludeBlock {node_id}");

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
            let mut messages = Vec::new();

            // Resolve the source into a fully qualified URL (including `file://` URL)
            let url = if source.starts_with("https://") || source.starts_with("http://") {
                source.to_string()
            } else {
                // Make the path relative to the home dir of execution
                let path = executor.home().join(source);
                ["file://", &path.to_string_lossy()].concat()
            };

            // Decode the URL
            let content: Option<Vec<Block>> = match codecs::from_url(
                &url,
                Some(DecodeOptions {
                    media_type: self.media_type.clone(),
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

            if let Some(mut content) = content {
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

            let messages = (!messages.is_empty()).then_some(messages);

            let ended = Timestamp::now();

            let status = execution_status(&messages);
            let required = execution_required(&status);
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
