use std::path::PathBuf;

use schema::{CompilationDigest, PromptBlock, replicate};

use crate::{prelude::*, state_digest};

impl Executable for PromptBlock {
    #[tracing::instrument(skip_all)]
    async fn compile(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();

        let state_digest = state_digest!(
            self.instruction_type,
            self.node_types,
            self.query,
            self.target
        );

        let compilation_digest = CompilationDigest::new(state_digest);

        if Some(&compilation_digest) == self.options.compilation_digest.as_ref() {
            tracing::trace!("Skipping compiling PromptBlock {node_id}");

            return WalkControl::Break;
        }

        tracing::trace!("Compiling PromptBlock {node_id}");

        // Infer prompt if appropriate
        if self.target.is_none()
            || self
                .target
                .as_ref()
                .map(|target| target.ends_with("?"))
                .unwrap_or_default()
        {
            if let Some(prompt) = prompts::infer(
                &self.instruction_type,
                &self.node_types,
                &self.query.as_deref(),
            )
            .await
            {
                let name = [&prompts::shorten(&prompt.name, &self.instruction_type), "?"].concat();
                self.target = Some(name.clone());
                executor.patch(&node_id, [set(NodeProperty::Target, name)]);
            }
        }

        // Populate prompt content so it is preview-able to the user
        let messages = if let Some(target) = &self.target {
            let target = prompts::expand(target, &self.instruction_type);
            match prompts::get(&target).await {
                Ok(prompt) => {
                    // Get the home directory of the prompt, needed at execution times
                    let dir = prompt.home().to_string_lossy().to_string();

                    // Replicate the content of the prompt so that the prompt block has different ids.
                    // Given that the same prompt could be used multiple times in a doc, if we don't
                    // do this there could be clashes.
                    let content = replicate(&prompt.content).unwrap_or_default();

                    // Set both here and via patch
                    self.options.directory = Some(dir.clone());
                    self.content = Some(content.clone());
                    executor.patch(
                        &node_id,
                        [
                            set(NodeProperty::Directory, dir),
                            // It is important to use `none` and `append` here because
                            // the latter retains node ids so they are the same as in `self.content`
                            // TODO: consider doing a merge rather than full replacement. Replacement
                            // seems to cause large, slow diffs in DOMs (do to a whole lot of new ids?)
                            none(NodeProperty::Content),
                            append(NodeProperty::Content, content),
                        ],
                    );

                    None
                }
                Err(error) => Some(vec![error_to_compilation_message(error)]),
            }
        } else {
            None
        };

        let execution_required =
            execution_required_digests(&self.options.execution_digest, &compilation_digest);

        self.options.compilation_digest = Some(compilation_digest.clone());
        self.options.execution_required = Some(execution_required);
        executor.patch(
            &node_id,
            [
                set(NodeProperty::CompilationDigest, compilation_digest),
                set(NodeProperty::CompilationMessages, messages),
                set(NodeProperty::ExecutionRequired, execution_required),
            ],
        );

        // Continue walk so that excerpts and citations in excerpts are compiled
        WalkControl::Continue
    }

    #[tracing::instrument(skip_all)]
    async fn prepare(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::trace!("Preparing PromptBlock {node_id}");

        // Set execution status
        if let Some(status) = executor.node_execution_status(
            self.node_type(),
            &node_id,
            &self.execution_mode,
            &self.options.execution_required,
        ) {
            self.options.execution_status = Some(status);
            executor.patch(&node_id, [set(NodeProperty::ExecutionStatus, status)]);
        }

        // Break walk because `content` is prepared in `execute` and
        // do not want headings, paragraphs etc to be added to context
        WalkControl::Break
    }

    #[tracing::instrument(skip_all)]
    async fn execute(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::debug!("Executing PromptBlock {node_id}");

        // Always execute, so mark as running
        let status = ExecutionStatus::Running;
        self.options.execution_status = Some(status);
        executor.patch(&node_id, [set(NodeProperty::ExecutionStatus, status)]);

        let started = Timestamp::now();
        let mut messages = Vec::new();

        // Execute content of prompt within the prompt's directory (so that includes work)
        let home = PathBuf::from(self.options.directory.as_deref().unwrap_or_default());
        executor.directory_stack.push(home);
        if let Err(error) = executor.compile_prepare_execute(&mut self.content).await {
            messages.push(error_to_execution_message("While executing prompt", error));
        }
        executor.directory_stack.pop();

        let ended = Timestamp::now();
        let messages = (!messages.is_empty()).then_some(messages);

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

        // Break walk because already walked over content with the new executor
        WalkControl::Break
    }
}
