use codec_markdown_trait::to_markdown;
use common::{futures::future, itertools::Itertools};
use schema::{AuthorRole, InstructionBlock, SuggestionStatus};

use crate::{assistant::execute_assistant, interrupt_impl, pending_impl, prelude::*};

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

        let started = Timestamp::now();

        let replicates = self.replicates.unwrap_or(1) as usize;

        let node_types = self
            .content
            .iter()
            .flatten()
            .map(|block| block.node_type().to_string())
            .collect_vec();

        let mut assistant =
            assistants::find(&self.assignee, &self.instruction_type, &Some(node_types))
                .await
                .unwrap();
        let assistant_name = assistant.name.clone();
        tracing::trace!("Using {assistant_name}");

        let content = self.content.as_ref().map(|content| to_markdown(content));

        execute_assistant(
            &mut assistant,
            &self.instruction_type,
            content,
            &executor.home,
        )
        .await
        .unwrap();

        let prompter: AuthorRole = assistant.clone().into();
        let system_prompt = assistants::render(assistant).await.unwrap();
        tracing::debug!("{assistant_name} rendered prompt:\n\n{system_prompt}");

        // Create a future for each replicate
        let mut futures = Vec::new();
        for _ in 0..replicates {
            let prompter = prompter.clone();
            let system_prompt = system_prompt.to_string();
            let instruction = self.clone();
            futures.push(async move {
                assistants::execute_instruction_block(
                    AuthorRole::default(),
                    prompter,
                    &system_prompt,
                    &instruction,
                )
                .await
            })
        }

        // Wait for all suggestions to be generated and collect them and any error messages
        let mut suggestions = self.suggestions.clone().unwrap_or_default();
        let mut messages = Vec::new();
        for result in future::join_all(futures).await {
            match result {
                Ok(suggestion) => suggestions.push(suggestion),
                Err(error) => messages.push(error_to_execution_message(
                    "While performing instruction",
                    error,
                )),
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
