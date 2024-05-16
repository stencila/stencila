use assistants::assistant::GenerateOptions;
use schema::{authorship, InstructionBlock};

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
            let (authors, suggestion, mut messages) = match assistants::execute_instruction(
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
                    Some(output.authors.clone()),
                    Some(output.to_suggestion_block(self.content.is_none())),
                    Vec::new(),
                ),
                Err(error) => (
                    None,
                    None,
                    vec![error_to_execution_message(
                        "While performing instruction",
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
                    [set(NodeProperty::Suggestion, suggestion.clone())],
                );

                // Execute the suggestion
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

        WalkControl::Continue
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
