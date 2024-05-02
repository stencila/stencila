use assistants::assistant::GenerateOptions;
use schema::InstructionInline;

use crate::{interrupt_impl, pending_impl, prelude::*};

impl Executable for InstructionInline {
    #[tracing::instrument(skip_all)]
    async fn pending(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::debug!("Pending InstructionInline {node_id}");

        if executor.should_execute_instruction_inline(&node_id, self) {
            tracing::debug!("Pending InstructionInline {node_id}");

            pending_impl!(executor, &node_id);
        }

        // Continue to mark executable nodes in `content` as pending
        WalkControl::Continue
    }

    #[tracing::instrument(skip_all)]
    async fn execute(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();

        if !executor.should_execute_instruction_inline(&node_id, self) {
            tracing::debug!("Skipping InstructionInline {node_id}");

            return WalkControl::Break;
        }

        tracing::trace!("Executing InstructionInline {node_id}");

        executor.patch(
            &node_id,
            [
                set(NodeProperty::ExecutionStatus, ExecutionStatus::Running),
                none(NodeProperty::ExecutionMessages),
            ],
        );

        if !self.messages.is_empty() {
            let started = Timestamp::now();

            // Get the `assistants` crate to execute this instruction
            let (mut suggestion, mut messages) = match assistants::execute_instruction(
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
                    Some(output.to_suggestion_inline(self.content.is_none())),
                    Vec::new(),
                ),
                Err(error) => (
                    None,
                    vec![error_to_execution_message("While performing instruction", error)],
                ),
            };

            // Execute the suggestion
            // TODO: This requires configurable rules around when, if at all, suggestions are executed.
            if let Err(error) = suggestion.walk_async(executor).await {
                messages.push(error_to_execution_message("While executing suggestion", error));
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
        tracing::debug!("Interrupting InstructionInline {node_id}");

        interrupt_impl!(self, executor, &node_id);

        // Continue to interrupt executable nodes in `content`
        WalkControl::Continue
    }
}
