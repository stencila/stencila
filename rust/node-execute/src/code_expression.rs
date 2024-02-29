use schema::CodeExpression;

use crate::prelude::*;

impl Executable for CodeExpression {
    #[tracing::instrument(skip_all)]
    async fn execute<'lt>(&mut self, executor: &mut Executor<'lt>) -> WalkControl {
        tracing::trace!("Executing CodeExpression {}", self.node_id());

        // Evaluate code (if it is not empty) in kernels
        let code = self.code.trim();
        if !code.is_empty() {
            let started = Timestamp::now();
            let (output, messages) = executor
                .kernels
                .evaluate(code, self.programming_language.as_deref())
                .await
                .unwrap_or_else(|error| {
                    (
                        Node::Null(Null),
                        vec![error_to_message("While evaluating expression", error)],
                    )
                });
            let ended = Timestamp::now();

            self.output = Some(Box::new(output));

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
