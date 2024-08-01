use schema::{CodeChunk, LabelType, NodeProperty};

use crate::{interrupt_impl, pending_impl, prelude::*, Phase};

impl Executable for CodeChunk {
    #[tracing::instrument(skip_all)]
    async fn compile(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::trace!("Compiling CodeChunk {node_id}");

        // Some code chunks should be executed during "compile" to
        // enable live updates (e.g. Graphviz, Mermaid)
        // TODO: consider having a way to specify which code chunks and/or
        // which kernels should execute at compile time (e.g. could have
        // a compile method on kernels)
        let lang = self
            .programming_language
            .as_ref()
            .map_or_else(String::new, |lang| lang.trim().to_lowercase());
        if lang == "dot" || lang == "graphviz" {
            return self.execute(executor).await;
        }

        let info = parsers::parse(
            &self.code,
            self.programming_language.as_deref().unwrap_or_default(),
        );

        let execution_required =
            execution_required_digests(&self.options.execution_digest, &info.compilation_digest);
        executor.patch(
            &node_id,
            [
                set(NodeProperty::CompilationDigest, info.compilation_digest),
                set(NodeProperty::ExecutionTags, info.execution_tags),
                set(NodeProperty::ExecutionRequired, execution_required),
            ],
        );

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

        WalkControl::Continue
    }

    #[tracing::instrument(skip_all)]
    async fn pending(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();

        if executor.should_execute(
            &node_id,
            &self.execution_mode,
            &self.options.compilation_digest,
            &self.options.execution_digest,
        ) {
            tracing::trace!("Pending CodeChunk {node_id}");
            pending_impl!(executor, &node_id);
        }

        WalkControl::Break
    }

    #[tracing::instrument(skip_all)]
    async fn execute(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();

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
