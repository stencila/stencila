use assistants::assistant::GenerateOptions;
use common::futures::future;
use schema::{authorship, InstructionBlock, SuggestionStatus};

use crate::{interrupt_impl, pending_impl, prelude::*};

impl Executable for InstructionBlock {
    #[tracing::instrument(skip_all)]
    async fn pending(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();

        if executor.should_execute_instruction(&node_id, &self.execution_mode) {
            tracing::trace!("Pending InstructionBlock {node_id}");

            pending_impl!(executor, &node_id);
        }

        // Continue to mark executable nodes in `content` and/or `suggestion` as pending
        WalkControl::Continue
    }

    #[tracing::instrument(skip_all)]
    async fn execute(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();

        // If any of the suggestions is in the executing node ids, it means that
        // this instruction needs to be re-executed
        // TODO: add the suggestion to the messages of the instruction, possibly
        // including feedback.
        let has_revisions =
            if let (Some(node_ids), Some(suggestions)) = (&executor.node_ids, &self.suggestions) {
                node_ids.iter().any(|node_id| {
                    suggestions
                        .iter()
                        .any(|suggestion| node_id == &suggestion.node_id())
                })
            } else {
                false
            };

        if !has_revisions && !executor.should_execute_instruction(&node_id, &self.execution_mode) {
            tracing::trace!("Skipping InstructionBlock {node_id}");

            // Continue to execute executable nodes in `content` and/or `suggestion`
            return WalkControl::Continue;
        }

        tracing::debug!("Executing InstructionBlock {node_id}");

        executor.patch(
            &node_id,
            [
                set(NodeProperty::ExecutionStatus, ExecutionStatus::Running),
                none(NodeProperty::ExecutionMessages),
            ],
        );

        if let Some(suggestions) = self.suggestions.as_mut() {
            // Remove suggestions that do not have a status or feedback
            suggestions.retain(|suggestion| {
                matches!(
                    suggestion.suggestion_status,
                    Some(SuggestionStatus::Accepted) | Some(SuggestionStatus::Rejected)
                ) || suggestion.feedback.is_some()
            });

            if suggestions.is_empty() {
                // Clear the suggestions if none left
                executor.patch(&node_id, [clear(NodeProperty::Suggestions)])
            } else {
                // Update the suggestions (to only those retained) and ensure they are visible
                executor.patch(
                    &node_id,
                    [set(NodeProperty::Suggestions, suggestions.clone())],
                );
            }
        }

        let replicates = self.replicates.unwrap_or(1) as usize;

        let started = Timestamp::now();

        // Create a future for each replicate
        let temperature = self
            .model
            .as_ref()
            .and_then(|model| model.temperature)
            .map(|temp| (temp as f32 / 100.).min(100.));
        let dry_run = executor.options.dry_run;
        let context = executor.context().await;
        let mut futures = Vec::new();
        for _ in 0..replicates {
            let instruction = self.clone();
            let context = context.clone();
            futures.push(async {
                let started = Timestamp::now();

                // Get the `assistants` crate to execute this instruction
                let (authors, mut suggestion, mut messages) = match assistants::execute_instruction(
                    instruction,
                    context,
                    GenerateOptions {
                        temperature,
                        dry_run,
                        ..Default::default()
                    },
                )
                .await
                {
                    Ok(output) => (
                        Some(output.authors.clone()),
                        Some(output.to_suggestion_block()),
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

                if let Some(suggestion) = suggestion.as_mut() {
                    // Apply authorship to the suggestion.
                    // Do this here, rather than by adding the authors to the patch
                    // so that the authors are not added to the instruction itself
                    if let Some(authors) = authors {
                        if let Err(error) = authorship(suggestion, authors) {
                            messages.push(error_to_execution_message(
                                "Unable to assign authorship to suggestion",
                                error,
                            ));
                        }
                    }

                    // Record execution time for the suggestion
                    let ended = Timestamp::now();
                    let duration = Some(execution_duration(&started, &ended));
                    let ended = Some(ended);
                    suggestion.execution_duration = duration;
                    suggestion.execution_ended = ended;
                    suggestion.suggestion_status = Some(SuggestionStatus::Proposed);
                }

                (suggestion, messages)
            })
        }

        // Wait for all suggestions to be generated and collect them and any messages
        let mut suggestions = self.suggestions.clone().unwrap_or_default();
        let mut messages = Vec::new();
        for mut replicate in future::join_all(futures).await {
            if let Some(suggestion) = replicate.0 {
                suggestions.push(suggestion);
            }
            messages.append(&mut replicate.1);
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
                set(NodeProperty::Suggestions, Some(suggestions)),
                set(NodeProperty::ExecutionStatus, status),
                set(NodeProperty::ExecutionRequired, required),
                set(NodeProperty::ExecutionMessages, messages),
                set(NodeProperty::ExecutionDuration, duration),
                set(NodeProperty::ExecutionEnded, ended),
                set(NodeProperty::ExecutionCount, count),
            ],
        );

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
