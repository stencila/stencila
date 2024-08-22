use schema::{CodeExpression, ExecutionMode};

use crate::{interrupt_impl, prelude::*, Phase};

impl Executable for CodeExpression {
    #[tracing::instrument(skip_all)]
    async fn compile(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::trace!("Compiling CodeExpression {node_id}");

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
                set(NodeProperty::ExecutionRequired, execution_required),
            ],
        );

        WalkControl::Continue
    }

    #[tracing::instrument(skip_all)]
    async fn prepare(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::trace!("Preparing CodeExpression {node_id}");

        if executor.should_execute(
            &node_id,
            &self.execution_mode.clone().or(Some(ExecutionMode::Always)),
            &self.options.compilation_digest,
            &self.options.execution_digest,
        ) {
            // Set the execution status to pending
            executor.patch(
                &node_id,
                [set(NodeProperty::ExecutionStatus, ExecutionStatus::Pending)],
            );
        }

        WalkControl::Break
    }

    #[tracing::instrument(skip_all)]
    async fn execute(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();

        if !executor.should_execute(
            &node_id,
            &self.execution_mode.clone().or(Some(ExecutionMode::Always)),
            &self.options.compilation_digest,
            &self.options.execution_digest,
        ) {
            tracing::trace!("Skipping CodeExpression {node_id}");
            return WalkControl::Break;
        }

        tracing::debug!("Executing CodeExpression {node_id}");

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

            let (output, messages) = executor
                .kernels
                .write()
                .await
                .evaluate(&self.code, self.programming_language.as_deref())
                .await
                .unwrap_or_else(|error| {
                    (
                        Node::Null(Null),
                        vec![error_to_execution_message(
                            "While evaluating expression",
                            error,
                        )],
                    )
                });

            let messages = (!messages.is_empty()).then_some(messages);

            let ended = Timestamp::now();

            let status = execution_status(&messages);
            let required = execution_required_status(&status);
            let duration = execution_duration(&started, &ended);
            let count = self.options.execution_count.unwrap_or_default() + 1;

            if matches!(executor.phase, Phase::ExecuteWithoutPatches) {
                self.output = Some(Box::new(output));
                self.options.execution_messages = messages.clone();
            } else {
                executor.patch(
                    &node_id,
                    [
                        set(NodeProperty::Output, output),
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
        tracing::debug!("Interrupting CodeExpression {node_id}");

        interrupt_impl!(self, executor, &node_id);

        WalkControl::Break
    }
}
