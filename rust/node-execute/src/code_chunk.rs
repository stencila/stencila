use schema::CodeChunk;

use crate::prelude::*;

impl Executable for CodeChunk {
    #[tracing::instrument(skip_all)]
    async fn execute<'lt>(&mut self, executor: &mut Executor<'lt>) -> WalkControl {
        tracing::trace!("Executing CodeChunk {}", self.node_id());

        // Execute code (if it is not empty) in kernels
        let code = self.code.trim();
        if !code.is_empty() {
            let started = Timestamp::now();
            let (outputs, messages) = executor
                .kernels
                .execute(code, self.programming_language.as_deref())
                .await
                .unwrap_or_else(|error| {
                    (
                        vec![],
                        vec![error_to_message("While executing code", error)],
                    )
                });
            let ended = Timestamp::now();

            self.outputs = if !outputs.is_empty() {
                Some(outputs)
            } else {
                None
            };

            self.options.execution_status = execution_status(&messages);
            self.options.execution_required = execution_required(&self.options.execution_status);
            self.options.execution_messages = execution_messages(messages);
            self.options.execution_duration = execution_duration(&started, &ended);
            self.options.execution_ended = Some(ended);
            self.options.execution_count.get_or_insert(0).add_assign(1);
        } else {
            self.options.execution_status = Some(ExecutionStatus::Empty);
            self.options.execution_required = Some(ExecutionRequired::No);
            self.options.execution_messages = None;
            self.options.execution_ended = None;
            self.options.execution_duration = None;
        }

        WalkControl::Break
    }
}
