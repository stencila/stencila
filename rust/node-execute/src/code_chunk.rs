use schema::{CodeChunk, LabelType, NodeProperty};

use crate::{interrupt_impl, prelude::*, Phase};

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

        let execution_required =
            execution_required_digests(&self.options.execution_digest, &info.compilation_digest);

        executor.patch(
            &node_id,
            [
                set(NodeProperty::CompilationDigest, info.compilation_digest.clone()),
                set(NodeProperty::ExecutionTags, info.execution_tags.clone()),
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
            // These need to be set here because they may be used in `self.execute`
            // and that method is called next, before the above patch is applied;
            self.options.compilation_digest = Some(info.compilation_digest);
            self.options.execution_tags = info.execution_tags;
            self.options.execution_required = Some(execution_required);

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

        if executor.should_execute(
            &node_id,
            &self.execution_mode,
            &self.options.compilation_digest,
            &self.options.execution_digest,
        ) {
            // Set the execution status to pending
            executor.patch(
                &node_id,
                [set(NodeProperty::ExecutionStatus, ExecutionStatus::Pending)],
            );
        }

        // Break the walk since none of the child nodes are executed
        WalkControl::Break
    }

    #[tracing::instrument(skip_all)]
    async fn execute(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();

        // Enter the code chunk context
        executor.document_context.code_chunks.enter();

        if !executor.should_execute(
            &node_id,
            &self.execution_mode,
            &self.options.compilation_digest,
            &self.options.execution_digest,
        ) {
            tracing::trace!("Skipping CodeChunk {node_id}");
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

            let (outputs, messages) = executor
                .kernels()
                .await
                .execute(&self.code, self.programming_language.as_deref())
                .await
                .unwrap_or_else(|error| {
                    (
                        Vec::new(),
                        vec![error_to_execution_message("While executing code", error)],
                    )
                });

            let outputs = (!outputs.is_empty()).then_some(outputs);
            let messages = (!messages.is_empty()).then_some(messages);

            let ended = Timestamp::now();

            let status = execution_status(&messages);
            let required = execution_required_status(&status);
            let duration = execution_duration(&started, &ended);
            let count = self.options.execution_count.unwrap_or_default() + 1;

            if matches!(executor.phase, Phase::ExecuteWithoutPatches) {
                self.outputs = outputs;
                self.options.execution_messages = messages.clone();
            } else {
                executor.patch(
                    &node_id,
                    [
                        set(NodeProperty::Outputs, outputs),
                        set(NodeProperty::ExecutionStatus, status.clone()),
                        set(NodeProperty::ExecutionRequired, required),
                        set(NodeProperty::ExecutionMessages, messages),
                        set(NodeProperty::ExecutionDuration, duration),
                        set(NodeProperty::ExecutionEnded, ended),
                        set(NodeProperty::ExecutionCount, count),
                        set(NodeProperty::ExecutionDigest, compilation_digest),
                    ],
                );
            }
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
