use codec_cbor::r#trait::CborCodec;
use codec_markdown_trait::{MarkdownCodec, MarkdownEncodeContext};
use codecs::Format;
use common::{
    futures::stream::{FuturesUnordered, StreamExt},
    itertools::Itertools,
    tokio,
};
use schema::{
    Author, AuthorRole, AuthorRoleAuthor, AuthorRoleName, CompilationDigest, ExecutionBounds,
    InstructionBlock, SoftwareApplication,
};

use crate::{interrupt_impl, prelude::*, state_digest};

impl Executable for InstructionBlock {
    #[tracing::instrument(skip_all)]
    async fn compile(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::trace!("Compiling InstructionBlock {node_id}");

        // Generate a compilation digest that captures the state of properties that
        // determine if a re-execution is required. The feedback on suggestions is
        // ignored because that would change the digest when a suggestion is deleted.
        let state_digest = state_digest!(
            self.instruction_type,
            self.message.to_cbor().unwrap_or_default().as_slice(),
            self.prompt.target,
            self.model_parameters
                .to_cbor()
                .unwrap_or_default()
                .as_slice()
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

        // Call `prompt.compile` directly because a `PromptBlock` that
        // is not a `Block::PromptBlock` variant is not walked over
        self.prompt.compile(executor).await;

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
        let mut model_ids = self.model_parameters.model_ids.clone().clone();
        let mut replicates = (self.model_parameters.replicates.unwrap_or(1) as usize).min(10);

        // If this is a revision (i.e. a retry, possibly with feedback already added to suggestions)
        // as indicated by previous suggestions being retained, then (a) set the number of replicates
        // to one, (b) use the same model as that which generated the active suggestion, and
        // (c) put the active suggestion last when sending to the model <<- TODO
        if executor.options.retain_suggestions {
            replicates = 1;

            if let (Some(index), Some(suggestions)) = (self.active_suggestion, &self.suggestions) {
                if let Some(suggestion) = suggestions.get(index as usize) {
                    model_ids = suggestion.authors.iter().flatten().find_map(|author| {
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
                            }) => Some(vec![id.clone()]),
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
        // TODO: reinstate use of node_types
        let _ = self
            .content
            .iter()
            .flatten()
            .map(|block| block.node_type().to_string())
            .collect_vec();

        // Execute the `PromptBlock`. The instruction context needs to
        // be set for the prompt context to be complete (i.e. include `instruction` variable)
        executor.instruction_context = Some((&*self).into());
        self.prompt.execute(executor).await;
        executor.instruction_context = None;

        // Render the `PromptBlock` into a system prompt
        let mut context = MarkdownEncodeContext::new(Some(Format::Markdown), Some(true));
        self.prompt.content.to_markdown(&mut context);
        let system_prompt = context.content;

        // Create an author role for the prompt
        let prompter = AuthorRole {
            last_modified: Some(Timestamp::now()),
            ..Default::default() // TODO: reinstate getting the author role for the prompt
                                 //..prompt.deref().clone().into()
        };

        // Get the authors of the instruction
        let mut instructors = Vec::new();
        for author in self.message.authors.iter().flatten() {
            instructors.push(AuthorRole {
                last_modified: Some(Timestamp::now()),
                ..author.clone().into_author_role(AuthorRoleName::Instructor)
            });
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
            if let Some(model_ids) = model_ids.clone() {
                // Apply the model id for revisions
                instruction.model_parameters.model_ids = Some(model_ids);
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
        let bounds = self.execution_bounds.clone().unwrap_or_default();
        while let Some(result) = futures.next().await {
            match result {
                Ok(mut suggestion) => {
                    executor.patch(
                        &node_id,
                        [push(NodeProperty::Suggestions, suggestion.clone())],
                    );

                    if !matches!(bounds, ExecutionBounds::Skip) {
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
