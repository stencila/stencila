use assistants::assistant::GenerateOptions;
use schema::{InstructionBlock, SuggestionBlockType};

use crate::{interrupt_impl, pending_impl, prelude::*};

impl Executable for InstructionBlock {
    #[tracing::instrument(skip_all)]
    async fn pending(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();

        if executor.should_execute_instruction_block(&node_id, self) {
            tracing::debug!("Pending InstructionBlock {node_id}");

            pending_impl!(executor, &node_id);
        }

        // Continue to mark executable nodes in `content` and/or `suggestion` as pending
        WalkControl::Continue
    }

    #[tracing::instrument(skip_all)]
    async fn execute(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();

        if !executor.should_execute_instruction_block(&node_id, self) {
            tracing::debug!("Skipping InstructionBlock {node_id}");

            // Continue to execute executable nodes in `content` and/or `suggestion`
            return WalkControl::Continue;
        }

        tracing::trace!("Executing InstructionBlock {node_id}");

        executor.replace_properties(
            &node_id,
            [
                (Property::ExecutionStatus, ExecutionStatus::Running.into()),
                (Property::ExecutionMessages, Value::None),
            ],
        );

        if !self.messages.is_empty() {
            let started = Timestamp::now();

            // Get the `assistants` crate to execute this instruction
            let (suggestion, mut messages) = match assistants::execute_instruction(
                self.clone(),
                executor.context().await,
                GenerateOptions {
                    dry_run: executor.options.dry_run,
                    ..Default::default()
                },
            )
            .await
            {
                Ok(output) => (
                    Some(output.to_suggestion_block(self.content.is_none())),
                    Vec::new(),
                ),
                Err(error) => (
                    None,
                    vec![error_to_message("While performing instruction", error)],
                ),
            };

            // Insert the suggestion into the store, so that it can be executed in
            // the next step (if so configured) and update the instruction status
            let mut suggestion: Option<SuggestionBlockType> = match executor
                .swap_property(&node_id, Property::Suggestion, suggestion.into())
                .await
            {
                Ok(suggestion) => suggestion,
                Err(error) => {
                    messages.push(error_to_message("While loading content", error));
                    None
                }
            };

            // Execute the suggestion
            // TODO: This requires configurable rules around when, if at all, suggestions are executed.
            if let Err(error) = suggestion.walk_async(executor).await {
                messages.push(error_to_message("While executing suggestion", error));
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
                    (Property::ExecutionStatus, status.into()),
                    (Property::ExecutionRequired, required.into()),
                    (Property::ExecutionMessages, messages.into()),
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
        tracing::debug!("Interrupting InstructionBlock {node_id}");

        interrupt_impl!(self, executor, &node_id);

        // Continue to interrupt executable nodes in `content`
        WalkControl::Continue
    }
}
