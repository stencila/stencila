use schema::{CodeChunk, LabelType, NodeProperty};

use crate::{interrupt_impl, prelude::*};

impl Executable for CodeChunk {
    #[tracing::instrument(skip_all)]
    async fn compile(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::trace!("Compiling CodeChunk {node_id}");

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

        let lang = self.programming_language.as_deref().unwrap_or_default();
        let info = parsers::parse(&self.code, lang);

        let mut execution_required =
            execution_required_digests(&self.options.execution_digest, &info.compilation_digest);

        // Check whether the kernel instance used last time is active in the kernels set
        if let Some(id) = &self.options.execution_instance {
            if !executor.kernels().await.has_instance(id).await {
                execution_required = ExecutionRequired::KernelRestarted;
            }
        }

        // These need to be set here because they may be used in `self.execute`
        // before the following patch is applied (below, or if `Executor.compile_prepare_execute`)
        // has been called.
        self.options.compilation_digest = Some(info.compilation_digest.clone());
        self.options.execution_tags = info.execution_tags.clone();
        self.options.execution_required = Some(execution_required.clone());

        executor.patch(
            &node_id,
            [
                set(NodeProperty::CompilationDigest, info.compilation_digest),
                set(NodeProperty::ExecutionTags, info.execution_tags),
                set(NodeProperty::ExecutionRequired, execution_required.clone()),
            ],
        );

        // Some code chunks should be executed during "compile" phase to
        // enable live updates (e.g. Graphviz, Mermaid)
        // TODO: consider having a way to specify which code chunks and/or
        // which kernels should execute at compile time (e.g. could have
        // a compile method on kernels)
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
            self.options.execution_status = Some(status.clone());
            executor.patch(&node_id, [set(NodeProperty::ExecutionStatus, status)]);
        }

        // Break the walk since none of the child nodes are executed
        WalkControl::Break
    }

    #[tracing::instrument(skip_all)]
    async fn execute(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();

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

            let (outputs, messages, instance) = executor
                .kernels()
                .await
                .execute(&self.code, self.programming_language.as_deref())
                .await
                .unwrap_or_else(|error| {
                    (
                        Vec::new(),
                        vec![error_to_execution_message("While executing code", error)],
                        String::new(),
                    )
                });

            let outputs = (!outputs.is_empty()).then_some(outputs);
            let messages = (!messages.is_empty()).then_some(messages);

            let ended = Timestamp::now();

            let status = execution_status(&messages);
            let kind = execution_kind(executor);
            let required = execution_required_status(&status);
            let duration = execution_duration(&started, &ended);
            let count = self.options.execution_count.unwrap_or_default() + 1;

            // Set properties that may be using in rendering
            self.outputs = outputs.clone();
            self.options.execution_messages = messages.clone();

            // Patch outputs using kernel as author if instance has changed
            if let Some(author) = executor
                .node_execution_instance_author(&instance, &self.options.execution_instance)
                .await
            {
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
                    set(NodeProperty::ExecutionKind, kind),
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
