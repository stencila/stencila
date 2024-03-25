use schema::{AutomaticExecution, CodeExpression};

use crate::{interrupt_impl, pending_impl, prelude::*};

impl Executable for CodeExpression {
    #[tracing::instrument(skip_all)]
    async fn compile(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::debug!("Compiling CodeExpression {node_id}");

        let info = parsers::parse(
            &self.code,
            self.programming_language.as_deref().unwrap_or_default(),
        );

        executor.replace_property(
            &node_id,
            Property::CompilationDigest,
            info.compilation_digest.into(),
        );

        WalkControl::Continue
    }

    #[tracing::instrument(skip_all)]
    async fn pending(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::debug!("Pending CodeExpression {node_id}");

        pending_impl!(executor, &node_id);

        WalkControl::Break
    }

    #[tracing::instrument(skip_all)]
    async fn execute(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();

        if !executor.should_execute_code(
            &node_id,
            &self.auto_exec.clone().or(Some(AutomaticExecution::Always)),
            &self.options.compilation_digest,
            &self.options.execution_digest,
        ) {
            tracing::debug!("Skipping CodeExpression {node_id}");

            return WalkControl::Break;
        }

        tracing::trace!("Executing CodeExpression {node_id}");

        executor.replace_properties(
            &node_id,
            [
                (Property::ExecutionStatus, ExecutionStatus::Running.into()),
                (Property::ExecutionMessages, Value::None),
            ],
        );

        let compilation_digest = self.options.compilation_digest.clone();

        let code = self.code.trim();
        if !code.is_empty() {
            let started = Timestamp::now();

            let (output, messages) = executor
                .kernels
                .write()
                .await
                .evaluate(code, self.programming_language.as_deref())
                .await
                .unwrap_or_else(|error| {
                    (
                        Node::Null(Null),
                        vec![error_to_message("While evaluating expression", error)],
                    )
                });

            let messages = (!messages.is_empty()).then_some(messages);

            let ended = Timestamp::now();

            let status = execution_status(&messages);
            let required = execution_required(&status);
            let duration = execution_duration(&started, &ended);
            let count = self.options.execution_count.unwrap_or_default() + 1;

            executor.replace_properties(
                &node_id,
                [
                    (Property::Output, output.into()),
                    (Property::ExecutionStatus, status.clone().into()),
                    (Property::ExecutionRequired, required.into()),
                    (Property::ExecutionMessages, messages.into()),
                    (Property::ExecutionDuration, duration.into()),
                    (Property::ExecutionEnded, ended.into()),
                    (Property::ExecutionCount, count.into()),
                    (Property::ExecutionDigest, compilation_digest.into()),
                ],
            );
        } else {
            executor.replace_properties(
                &node_id,
                [
                    (Property::ExecutionStatus, ExecutionStatus::Empty.into()),
                    (Property::ExecutionRequired, ExecutionRequired::No.into()),
                    (Property::ExecutionMessages, Value::None),
                    (Property::ExecutionDuration, Value::None),
                    (Property::ExecutionEnded, Value::None),
                    (Property::ExecutionDigest, compilation_digest.into()),
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
