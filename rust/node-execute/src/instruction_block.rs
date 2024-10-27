use std::ops::Deref;

use codec_cbor::r#trait::CborCodec;
use codec_markdown_trait::{MarkdownCodec, MarkdownEncodeContext};
use codecs::Format;
use common::{
    futures::stream::{FuturesUnordered, StreamExt},
    itertools::Itertools,
    tokio,
};
use schema::{
    Author, AuthorRole, AuthorRoleAuthor, AuthorRoleName, CompilationDigest, InstructionBlock,
    InstructionModel, PromptBlock, SoftwareApplication,
};

use crate::{interrupt_impl, prelude::*};

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
            self.prompt.clone().unwrap_or_default().as_bytes(),
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
    async fn prepare(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::trace!("Preparing InstructionBlock {node_id}");

        // Set execution status
        if let Some(status) = executor.node_execution_status(
            self.node_type(),
            &node_id,
            &self.execution_mode,
            &self.options.compilation_digest,
            &self.options.execution_digest,
        ) {
            self.options.execution_status = Some(status.clone());
            executor.patch(&node_id, [set(NodeProperty::ExecutionStatus, status)]);
        }

        // Continue to mark executable nodes in `content` and/or `suggestion` as pending
        WalkControl::Continue
    }

    #[tracing::instrument(skip_all)]
    async fn execute(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();

        // Get options which may be overridden if this is a revision
        // Note: to avoid accidentally generating many replicates, hard code maximum 10 here
        let mut replicates = (self.replicates.unwrap_or(1) as usize).min(10);
        let mut model_id_pattern = self
            .model
            .as_ref()
            .and_then(|model| model.id_pattern.clone())
            .clone();

        // If this is a revision (i.e. a retry, possibly with feedback already added to suggestions)
        // as indicated by previous suggestions being retained, then (a) set the number of replicates
        // to one, (b) use the same model as that which generated the active suggestion, and
        // (c) put the active suggestion last when sending to the model <<- TODO
        if executor.options.retain_suggestions {
            replicates = 1;

            if let (Some(index), Some(suggestions)) = (self.active_suggestion, &self.suggestions) {
                if let Some(suggestion) = suggestions.get(index as usize) {
                    model_id_pattern = suggestion.authors.iter().flatten().find_map(|author| {
                        match author {
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
                        }
                    });
                }
            }
        };

        if !matches!(
            self.options.execution_status,
            Some(ExecutionStatus::Pending)
        ) {
            tracing::trace!("Skipping InstructionBlock {node_id}");

            // Continue to execute executable nodes in `content` and/or `suggestions`
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
        let mut messages = Vec::new();

        // Determine the types of nodes in the content of the instruction
        let node_types = self
            .content
            .iter()
            .flatten()
            .map(|block| block.node_type().to_string())
            .collect_vec();

        // Select the best prompt for the instruction
        let prompt = match prompts::select(
            &self.instruction_type,
            &self.message,
            &self.prompt,
            &Some(node_types),
        )
        .await
        {
            Ok(prompt) => prompt,
            Err(error) => {
                messages.push(error_to_execution_message("While selecting prompt", error));

                executor.patch(
                    &node_id,
                    [
                        set(NodeProperty::ExecutionStatus, ExecutionStatus::Errors),
                        set(
                            NodeProperty::ExecutionRequired,
                            ExecutionRequired::ExecutionFailed,
                        ),
                        set(NodeProperty::ExecutionMessages, messages),
                    ],
                );

                return WalkControl::Continue;
            }
        };
        let prompt_id = prompt.id.clone().unwrap_or_default();

        // Create a new `PromptBlock` to render the prompt and patch it to `prompt_provided`
        // so that when it is executed it gets patched
        let mut prompt_block = PromptBlock::new(prompt_id);
        executor.patch(
            &node_id,
            [set(NodeProperty::PromptProvided, prompt_block.clone())],
        );

        // Execute the `PromptBlock`. The instruction context needs to
        // be set for the prompt context to be complete (i.e. include `instruction` variable)
        executor.instruction_context = Some((&*self).into());
        prompt_block.execute(executor).await;
        executor.instruction_context = None;

        // Render the `PromptBlock` into a system prompt
        let mut context = MarkdownEncodeContext::new(Some(Format::Markdown), Some(true));
        prompt_block.content.to_markdown(&mut context);
        let system_prompt = context.content;

        // Create an author role for the prompt
        let prompter = AuthorRole {
            last_modified: Some(Timestamp::now()),
            ..prompt.deref().clone().into()
        };

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

        // Unless specified, clear existing suggestions
        if !executor.options.retain_suggestions {
            executor.patch(&node_id, [none(NodeProperty::Suggestions)]);
        }

        // Create a future for each replicate
        let mut futures = FuturesUnordered::new();
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

        // Wait for each future, adding the suggestion (or error message) to the instruction
        // as it arrives, and then (optionally) executing the suggestion
        let recursion = self.recursion.as_deref().unwrap_or_default();
        let run = recursion.contains("run") && !recursion.contains("!run");
        while let Some(result) = futures.next().await {
            match result {
                Ok(mut suggestion) => {
                    executor.patch(
                        &node_id,
                        [push(NodeProperty::Suggestions, suggestion.clone())],
                    );

                    if run {
                        let mut fork = executor.fork_for_all();
                        tokio::spawn(async move {
                            if let Err(error) = fork.compile_prepare_execute(&mut suggestion).await
                            {
                                tracing::error!("While executing suggestion: {error}");
                            }
                        });
                    }
                }
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
