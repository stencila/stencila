use schema::CodeChunk;

use crate::{interrupt_impl, pending_impl, prelude::*};

impl Executable for CodeChunk {
    #[tracing::instrument(skip_all)]
    async fn pending(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::debug!("Pending CodeChunk {node_id}");

        pending_impl!(executor, &node_id);

        WalkControl::Break
    }

    #[tracing::instrument(skip_all)]
    async fn execute(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();

        if !executor.should_execute_code() {
            tracing::debug!("Skipping CodeChunk {node_id}");

            return WalkControl::Break;
        }

        tracing::trace!("Executing CodeChunk {node_id}");

        executor.replace_properties(
            &node_id,
            [
                (Property::ExecutionStatus, ExecutionStatus::Running.into()),
                (Property::ExecutionMessages, Value::None),
            ],
        );

        let code = self.code.trim();
        let add_to_context = if !code.is_empty() {
            let started = Timestamp::now();

            let (outputs, messages) = executor
                .kernels()
                .await
                .execute(code, self.programming_language.as_deref())
                .await
                .unwrap_or_else(|error| {
                    (
                        Vec::new(),
                        vec![error_to_message("While executing code", error)],
                    )
                });

            let outputs = (!outputs.is_empty()).then_some(outputs);
            let messages = (!messages.is_empty()).then_some(messages);

            let ended = Timestamp::now();

            let status = execution_status(&messages);
            let required = execution_required(&status);
            let duration = execution_duration(&started, &ended);
            let count = self.options.execution_count.unwrap_or_default() + 1;

            executor.replace_properties(
                &node_id,
                [
                    (Property::Outputs, outputs.into()),
                    (Property::ExecutionStatus, status.clone().into()),
                    (Property::ExecutionRequired, required.into()),
                    (Property::ExecutionMessages, messages.into()),
                    (Property::ExecutionDuration, duration.into()),
                    (Property::ExecutionEnded, ended.into()),
                    (Property::ExecutionCount, count.into()),
                ],
            );

            matches!(status, ExecutionStatus::Succeeded)
        } else {
            executor.replace_properties(
                &node_id,
                [
                    (Property::Outputs, Value::None),
                    (Property::ExecutionStatus, ExecutionStatus::Empty.into()),
                    (Property::ExecutionRequired, ExecutionRequired::No.into()),
                    (Property::ExecutionMessages, Value::None),
                    (Property::ExecutionDuration, Value::None),
                    (Property::ExecutionEnded, Value::None),
                ],
            );

            false
        };

        // If the code chunk succeeded add it to the context
        if add_to_context {
            executor.context.push_code_chunk(self);
        }

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
