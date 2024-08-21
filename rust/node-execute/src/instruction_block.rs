use codec_cbor::r#trait::CborCodec;
use codec_markdown_trait::to_markdown;
use common::{eyre::Result, futures::future, itertools::Itertools};
use prompts::Context;
use schema::{
    Author, AuthorRole, AuthorRoleAuthor, AuthorRoleName, Block, CompilationDigest,
    InstructionBlock, InstructionMessage, InstructionModel, InstructionType, SoftwareApplication,
};

use crate::{interrupt_impl, pending_impl, prelude::*, prompt::execute_prompt};

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

        // Get options which may be overridden if this is a suggestion
        // Note: to avoid accidentally generating many replicates, hard code maximum 10 here
        let mut replicates = (self.replicates.unwrap_or(1) as usize).min(10);
        let mut model_id_pattern = self
            .model
            .as_ref()
            .and_then(|model| model.id_pattern.clone())
            .clone();

        // If any of this instructions suggestions is in `executor.node_ids`, it indicates
        // that a revision of the suggestion is required (i.e only the one replicate,
        // and the model that generated the suggestion should be used again).
        let is_revision =
            if let (Some(node_ids), Some(suggestions)) = (&executor.node_ids, &self.suggestions) {
                if let Some(suggestion) = suggestions.iter().find(|suggestion| {
                    node_ids
                        .iter()
                        .any(|node_id| node_id == &suggestion.node_id())
                }) {
                    replicates = 1;
                    model_id_pattern =
                        suggestion
                            .authors
                            .iter()
                            .flatten()
                            .find_map(|author| match author {
                                // Gets the first generator author having an id
                                Author::AuthorRole(AuthorRole {
                                    role_name: AuthorRoleName::Generator,
                                    author:
                                        AuthorRoleAuthor::SoftwareApplication(SoftwareApplication {
                                            id: Some(id),
                                            ..
                                        }),
                                    ..
                                }) => Some(id.clone()),
                                _ => None,
                            });
                    true
                } else {
                    false
                }
            } else {
                false
            };

        if !is_revision
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

        // Find an prompt and generate a system prompt
        let (prompter, system_prompt) = match generate_system_prompt(
            &self.instruction_type,
            &self.message,
            &self.assignee,
            &self.content,
            executor.context().await,
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

        tracing::debug!("Using prompt:\n\n{system_prompt}");

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
        let mut futures = Vec::new();
        for _ in 0..replicates {
            // TODO: rather than repeating all this prep work to create a model task
            // within `prompts::execute_instruction_block` it could be done
            // once, and then clones and moved to each instruction.
            let instructors = instructors.clone();
            let prompter = prompter.clone();
            let system_prompt = system_prompt.to_string();
            let mut instruction = self.clone();
            let dry_run = executor.options.dry_run;
            if let Some(id_pattern) = model_id_pattern.clone() {
                // Apply the model id for revisions
                let id_pattern = Some(id_pattern);
                instruction.model = Some(Box::new(match instruction.model {
                    Some(model) => InstructionModel {
                        id_pattern,
                        ..*model
                    },
                    None => InstructionModel {
                        id_pattern,
                        ..Default::default()
                    },
                }))
            };
            futures.push(async move {
                prompts::execute_instruction_block(
                    instructors,
                    prompter,
                    &system_prompt,
                    &instruction,
                    dry_run,
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
                    "While executing instruction",
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
 * Find an prompt for the instruction and render a prompt from it
 */
async fn generate_system_prompt(
    instruction_type: &InstructionType,
    message: &Option<InstructionMessage>,
    assignee: &Option<String>,
    content: &Option<Vec<Block>>,
    context: &Context,
) -> Result<(AuthorRole, String)> {
    let node_types = content
        .iter()
        .flatten()
        .map(|block| block.node_type().to_string())
        .collect_vec();

    let mut prompt = prompts::find(instruction_type, message, assignee, &Some(node_types)).await?;
    let prompter = AuthorRole {
        last_modified: Some(Timestamp::now()),
        ..prompt.clone().into()
    };

    let content = content.as_ref().map(to_markdown);
    execute_prompt(&mut prompt, instruction_type, content, context).await?;
    let prompt = prompts::render(prompt).await.unwrap();

    Ok((prompter, prompt))
}
