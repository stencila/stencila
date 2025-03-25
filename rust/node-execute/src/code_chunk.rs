use schema::{CodeChunk, ExecutionBounds, LabelType, NodeProperty};

use crate::{interrupt_impl, prelude::*};

impl Executable for CodeChunk {
    #[tracing::instrument(skip_all)]
    async fn compile(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::trace!("Compiling CodeChunk {node_id}");

        // Update label if necessary
        if let Some(label_type) = &self.label_type {
            let label = match label_type {
                LabelType::FigureLabel => {
                    executor.figure_count += 1;
                    executor.figure_count.to_string()
                }
                LabelType::TableLabel => {
                    executor.table_count += 1;
                    executor.table_count.to_string()
                }
            };
            if self.label_automatically.unwrap_or(true) && Some(&label) != self.label.as_ref() {
                executor.patch(&node_id, [set(NodeProperty::Label, label)]);
            }
        }

        // Get the programming language, falling back to using the executor's current language
        let lang = executor.programming_language(&self.programming_language);

        // Parse the code to determine if it or the language has changed since last time
        let info = parsers::parse(&self.code, &lang, &self.options.compilation_digest);

        // Add code to the linting context
        executor.linting_code(&node_id, &self.code.to_string(), &lang, info.changed.yes());

        // Return early if no change
        if info.changed.no() {
            tracing::trace!("Skipping compiling CodeChunk {node_id}");

            return WalkControl::Break;
        }

        let mut execution_required =
            execution_required_digests(&self.options.execution_digest, &info.compilation_digest);

        // Check whether the kernel instance used last time is active in the kernels set (if not forked)
        if let (Some(ExecutionBounds::Main), Some(id)) = (
            &self.options.execution_bounded,
            &self.options.execution_instance,
        ) {
            if !executor.kernels().await.has_instance(id).await {
                execution_required = ExecutionRequired::KernelRestarted;
            }
        }

        // These need to be set here because they may be used in `self.execute`
        // before the following patch is applied (below, or if `Executor.compile_prepare_execute`)
        // has been called.
        self.options.compilation_digest = Some(info.compilation_digest.clone());
        self.options.execution_tags = info.execution_tags.clone();
        self.options.execution_required = Some(execution_required);

        executor.patch(
            &node_id,
            [
                set(NodeProperty::CompilationDigest, info.compilation_digest),
                set(NodeProperty::ExecutionTags, info.execution_tags),
                set(NodeProperty::ExecutionRequired, execution_required),
            ],
        );

        // Some code chunks should be executed during "compile" phase to
        // enable live updates (e.g. Graphviz, Mermaid)
        // TODO: consider having a way to specify which code chunks and/or
        // which kernels should execute at compile time (e.g. could have
        // a compile method on kernels)
        if let Some(lang) = lang {
            if !matches!(execution_required, ExecutionRequired::No)
                && matches!(
                    lang.trim().to_lowercase().as_str(),
                    "dot" | "graphviz" | "mermaid"
                )
            {
                // Need to set execution status to pending so avoid early return from
                // the execute methods
                self.options.execution_status = Some(ExecutionStatus::Pending);
                self.execute(executor).await;
            }
        }

        WalkControl::Continue
    }

    #[tracing::instrument(skip_all)]
    async fn prepare(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::trace!("Preparing CodeChunk {node_id}");

        // Add code chunk to document context
        executor.document_context.code_chunks.push((&*self).into());

        // Set execution status
        if let Some(status) = executor.node_execution_status(
            self.node_type(),
            &node_id,
            &self.execution_mode,
            &self.options.compilation_digest,
            &self.options.execution_digest,
        ) {
            self.options.execution_status = Some(status);
            executor.patch(&node_id, [set(NodeProperty::ExecutionStatus, status)]);
        }

        // Break the walk since none of the child nodes are executed
        WalkControl::Break
    }

    #[tracing::instrument(skip_all)]
    async fn execute(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();

        // Get the programming language, falling back to using the executor's current language
        let lang = executor.programming_language(&self.programming_language);

        // Add code to the linting context regardless of whether executed.
        // Do this during execution phase, for code chunks only,
        // because needed for linting code in chats and instructions which is generated during the
        // execute phase. If this is not done, then the linting context, lacks the variable declarations
        // imports etc in this code
        executor.linting_code(&node_id, &self.code.to_string(), &lang, true);

        // Enter the code chunk context
        executor.document_context.code_chunks.enter();

        if !matches!(
            self.options.execution_status,
            Some(ExecutionStatus::Pending)
        ) {
            tracing::trace!("Skipping CodeChunk {node_id}");

            // Exit the code chunk context
            executor.document_context.code_chunks.exit();

            return WalkControl::Break;
        }

        tracing::debug!("Executing CodeChunk {node_id}");

        executor.patch(
            &node_id,
            [
                set(NodeProperty::ExecutionStatus, ExecutionStatus::Running),
                none(NodeProperty::ExecutionMessages),
            ],
        );

        let compilation_digest = self.options.compilation_digest.clone();

        if !self.code.trim().is_empty() {
            let started = Timestamp::now();

            // Get the kernels to execute within, based on the the execution bounds
            let (kernels, message, bounded) = match self.execution_bounds {
                Some(ExecutionBounds::Main) | None => (Some(executor.kernels.clone()), None, None),
                Some(bounds) => match executor.replicate_kernels(bounds, lang.as_deref()).await {
                    Ok(kernels) => (Some(kernels), None, Some(bounds)),
                    Err(error) => (
                        None,
                        Some(error_to_execution_message(
                            "While replicating kernels",
                            error,
                        )),
                        None,
                    ),
                },
            };

            let (outputs, messages, instance) = if let Some(kernels) = kernels {
                kernels
                    .write()
                    .await
                    .execute(&self.code, lang.as_deref())
                    .await
                    .unwrap_or_else(|error| {
                        (
                            Vec::new(),
                            vec![error_to_execution_message("While executing code", error)],
                            String::new(),
                        )
                    })
            } else {
                (
                    Vec::new(),
                    message.map(|message| vec![message]).unwrap_or_default(),
                    String::new(),
                )
            };

            let outputs = (!outputs.is_empty()).then_some(outputs);
            let messages = (!messages.is_empty()).then_some(messages);

            let ended = Timestamp::now();

            let status = execution_status(&messages);
            let required = execution_required_status(&status);
            let duration = execution_duration(&started, &ended);
            let count = self.options.execution_count.unwrap_or_default() + 1;

            // Set properties that may be using in rendering
            self.outputs = outputs.clone();
            self.options.execution_messages = messages.clone();

            // Patch outputs using kernel instance as `AuthorRole` if possible
            if let Some(author) = executor.node_execution_author_role(&instance).await {
                executor.patch_with_authors(
                    &node_id,
                    vec![author],
                    [set(NodeProperty::Outputs, outputs)],
                );
            } else {
                executor.patch(&node_id, [set(NodeProperty::Outputs, outputs)]);
            }

            executor.patch(
                &node_id,
                [
                    set(NodeProperty::ExecutionStatus, status),
                    set(NodeProperty::ExecutionInstance, instance),
                    set(NodeProperty::ExecutionBounded, bounded),
                    set(NodeProperty::ExecutionRequired, required),
                    set(NodeProperty::ExecutionMessages, messages),
                    set(NodeProperty::ExecutionDuration, duration),
                    set(NodeProperty::ExecutionEnded, ended),
                    set(NodeProperty::ExecutionCount, count),
                    set(NodeProperty::ExecutionDigest, compilation_digest),
                ],
            );
        } else {
            executor.patch(
                &node_id,
                [
                    none(NodeProperty::Outputs),
                    set(NodeProperty::ExecutionStatus, ExecutionStatus::Empty),
                    set(NodeProperty::ExecutionRequired, ExecutionRequired::No),
                    none(NodeProperty::ExecutionMessages),
                    none(NodeProperty::ExecutionDuration),
                    none(NodeProperty::ExecutionEnded),
                    set(NodeProperty::ExecutionDigest, compilation_digest),
                ],
            );
        };

        // Exit the code chunk context
        executor.document_context.code_chunks.exit();

        WalkControl::Break
    }

    #[tracing::instrument(skip_all)]
    async fn interrupt(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::debug!("Interrupting CodeChunk {node_id}");

        interrupt_impl!(self, executor, &node_id);

        WalkControl::Break
    }
}
