use codecs::DecodeOptions;
use schema::{Block, IncludeBlock};

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
        tracing::trace!("Executing IncludeBlock {node_id}");

        executor.replace_properties(
            &node_id,
            [
                (Property::ExecutionStatus, ExecutionStatus::Running.into()),
                (Property::ExecutionMessages, Value::None),
            ],
        );

        let started = Timestamp::now();

        // Include the source (if it is not empty)
        let source = self.source.trim();
        if !source.is_empty() {
            let mut messages = Vec::new();

            // Resolve the source into a URL (including `file://` URL)
            let url = source; // TODO

            // Decode the URL
            let mut included: Option<Vec<Block>> = match codecs::from_url(
                url,
                Some(DecodeOptions {
                    media_type: self.media_type.clone(),
                    ..Default::default()
                }),
            )
            .await
            {
                Ok(_node) => {
                    // Transform the decoded source into a blocks
                    let blocks = vec![]; // TODO node.into();
                    Some(blocks)
                }
                Err(error) => {
                    messages.push(error_to_message("While executing code", error));
                    None
                }
            };

            // TODO: Implement sub-selecting from included based on `select`

            // Execute the included content
            if let Err(error) = included.walk_async(executor).await {
                messages.push(error_to_message("While executing included content", error));
            }

            let messages = (!messages.is_empty()).then_some(messages);

            let ended = Timestamp::now();

            let status = execution_status(&messages);
            let required = execution_required(&status);
            let duration = execution_duration(&started, &ended);
            let count = self.options.execution_count.unwrap_or_default() + 1;

            executor.replace_properties(
                &node_id,
                [
                    (Property::Content, included.into()),
                    (Property::ExecutionStatus, status.into()),
                    (Property::ExecutionRequired, required.into()),
                    (Property::ExecutionDuration, duration.into()),
                    (Property::ExecutionEnded, ended.into()),
                    (Property::ExecutionCount, count.into()),
                ],
            );
        } else {
            executor.replace_properties(
                &node_id,
                [
                    (Property::ExecutionStatus, ExecutionStatus::Empty.into()),
                    (Property::ExecutionRequired, ExecutionRequired::No.into()),
                    (Property::ExecutionDuration, Value::None),
                    (Property::ExecutionEnded, Value::None),
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
