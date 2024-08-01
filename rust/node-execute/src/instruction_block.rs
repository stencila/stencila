use std::path::Path;

use codec_cbor::r#trait::CborCodec;
use codec_markdown_trait::to_markdown;
use common::{eyre::Result, futures::future, itertools::Itertools};
use schema::{
    AuthorRole, AuthorRoleName, Block, CompilationDigest, InstructionBlock, InstructionType,
    SuggestionStatus,
};

use crate::{assistant::execute_assistant, interrupt_impl, pending_impl, prelude::*};

impl Executable for InstructionBlock {
    #[tracing::instrument(skip_all)]
    async fn compile(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::trace!("Compiling InstructionBlock {node_id}");

        // Generate a compilation digest that captures the state of properties that
        // determine if a re-execution is required. The feedback on suggestions is
        // ignored because that would change the digest when a suggestion is deleted.
        let mut state_digest = 0u64;
        add_to_digest(
            &mut state_digest,
            self.instruction_type.to_string().as_bytes(),
        );
        add_to_digest(
            &mut state_digest,
            self.message.to_cbor().unwrap_or_default().as_slice(),
        );
        add_to_digest(
            &mut state_digest,
            self.assignee.clone().unwrap_or_default().as_bytes(),
        );
        add_to_digest(
            &mut state_digest,
            self.model.to_cbor().unwrap_or_default().as_slice(),
        );
        add_to_digest(
            &mut state_digest,
            &self.replicates.unwrap_or(1).to_be_bytes(),
        );

        let compilation_digest = CompilationDigest::new(state_digest);
        let execution_required =
            execution_required_digests(&self.options.execution_digest, &compilation_digest);
        executor.patch(
            &node_id,
            [
                set(NodeProperty::CompilationDigest, compilation_digest),
                set(NodeProperty::ExecutionRequired, execution_required),
            ],
        );

        WalkControl::Continue
    }

    #[tracing::instrument(skip_all)]
    async fn pending(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();

        if executor.should_execute_instruction(
            &node_id,
            &self.execution_mode,
            &self.options.compilation_digest,
            &self.options.execution_digest,
        ) {
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
        let executing_a_suggestion =
            if let (Some(node_ids), Some(suggestions)) = (&executor.node_ids, &self.suggestions) {
                node_ids.iter().any(|node_id| {
                    suggestions
                        .iter()
                        .any(|suggestion| node_id == &suggestion.node_id())
                })
            } else {
                false
            };

        if !executing_a_suggestion
            && !executor.should_execute_instruction(
                &node_id,
                &self.execution_mode,
                &self.options.compilation_digest,
                &self.options.execution_digest,
            )
        {
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

        let started = Timestamp::now();

        // Clean up existing suggestions and send related patches
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

        // Find an assistant and generate a system prompt
        let (prompter, system_prompt) = match generate_system_prompt(
            &self.assignee,
            &self.instruction_type,
            &self.content,
            &executor.home,
        )
        .await
        {
            Ok(result) => result,
            Err(error) => {
                executor.patch(
                    &node_id,
                    [
                        set(NodeProperty::ExecutionStatus, ExecutionStatus::Exceptions),
                        set(
                            NodeProperty::ExecutionRequired,
                            ExecutionRequired::ExecutionFailed,
                        ),
                        set(
                            NodeProperty::ExecutionMessages,
                            vec![error_to_execution_message(
                                "While rendering prompt for instruction",
                                error,
                            )],
                        ),
                    ],
                );

                return WalkControl::Break;
            }
        };

        tracing::debug!("Using {prompter:?} prompt:\n\n{system_prompt}");

        // Get the authors of the instruction
        let mut instructors = Vec::new();
        if let Some(message) = &self.message {
            for author in message.authors.iter().flatten() {
                instructors.push(AuthorRole {
                    last_modified: Some(Timestamp::now()),
                    ..author.clone().into_author_role(AuthorRoleName::Instructor)
                });
            }
        }

        // Create a future for each replicate
        let replicates = self.replicates.unwrap_or(1) as usize;
        let mut futures = Vec::new();
        for _ in 0..replicates {
            let instructors = instructors.clone();
            let prompter = prompter.clone();
            let system_prompt = system_prompt.to_string();
            let instruction = self.clone();
            futures.push(async move {
                assistants::execute_instruction_block(
                    instructors,
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
        let compilation_digest = self.options.compilation_digest.clone();

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
                set(NodeProperty::ExecutionDigest, compilation_digest),
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

/**
 * Find an assistant for the instruction and render a prompt from it
 */
async fn generate_system_prompt(
    assignee: &Option<String>,
    instruction_type: &InstructionType,
    content: &Option<Vec<Block>>,
    home: &Path,
) -> Result<(AuthorRole, String)> {
    let node_types = content
        .iter()
        .flatten()
        .map(|block| block.node_type().to_string())
        .collect_vec();

    let mut assistant = assistants::find(&assignee, instruction_type, &Some(node_types)).await?;
    let prompter = AuthorRole {
        last_modified: Some(Timestamp::now()),
        ..assistant.clone().into()
    };

    let content = content.as_ref().map(to_markdown);
    execute_assistant(&mut assistant, instruction_type, content, home).await?;
    let prompt = assistants::render(assistant).await.unwrap();

    Ok((prompter, prompt))
}
