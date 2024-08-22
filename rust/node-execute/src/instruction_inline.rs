use schema::InstructionInline;

use crate::{interrupt_impl, prelude::*};

impl Executable for InstructionInline {
    #[tracing::instrument(skip_all)]
    async fn prepare(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::trace!("Preparing InstructionInline {node_id}");

        if executor.should_execute_instruction(&node_id, &self.execution_mode, &None, &None) {
            // Set the execution status to pending
            executor.patch(
                &node_id,
                [set(NodeProperty::ExecutionStatus, ExecutionStatus::Pending)],
            );
        }

        // Continue to mark executable nodes in `content` as pending
        WalkControl::Continue
    }

    #[tracing::instrument(skip_all)]
    async fn execute(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();

        if !executor.should_execute_instruction(&node_id, &self.execution_mode, &None, &None) {
            tracing::trace!("Skipping InstructionInline {node_id}");

            return WalkControl::Break;
        }

        tracing::debug!("Executing InstructionInline {node_id}");

        executor.patch(
            &node_id,
            [
                set(NodeProperty::ExecutionStatus, ExecutionStatus::Running),
                none(NodeProperty::ExecutionMessages),
            ],
        );

        let _started = Timestamp::now();

        // Get the `assistants` crate to execute this instruction
        /*
        TODO: reinstate based on approach for instruction blocks

        let (authors, suggestion, mut messages) = match assistants::execute_instruction(
            self.clone(),
            executor.context().await,
            ModelTaskOptions {
                dry_run: executor.options.dry_run,
                ..Default::default()
            },
        )
        .await
        {
            Ok(output) => (
                Some(output.authors.clone()),
                Some(output.to_suggestion_inline()),
                Vec::new(),
            ),
            Err(error) => (
                None,
                None,
                vec![error_to_execution_message(
                    "While executing instruction",
                    error,
                )],
            ),
        };


        if let Some(mut suggestion) = suggestion {
            // Apply authorship to the suggestion.
            // Do this here, rather than by adding the authors to the patch
            // so that the authors are not added to the instruction itself
            if let Some(authors) = authors {
                if let Err(error) = authorship(&mut suggestion, authors) {
                    messages.push(error_to_execution_message(
                        "Unable to assign authorship to suggestion",
                        error,
                    ));
                }
            }

            // Set the suggestion
            executor.patch(
                &node_id,
                [push(NodeProperty::Suggestions, suggestion.clone())],
            );

            // Execute the suggestion
            // TODO: This requires configurable rules around when, if at all, suggestions are executed.
            if let Err(error) = suggestion.walk_async(executor).await {
                messages.push(error_to_execution_message(
                    "While executing suggestion",
                    error,
                ));
            }
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
        */

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
