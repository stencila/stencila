use schema::{IfBlock, IfBlockClause};

use crate::prelude::*;

impl Executable for IfBlock {
    #[tracing::instrument(skip_all)]
    async fn execute<'lt>(&mut self, executor: &mut Executor<'lt>) -> WalkControl {
        tracing::trace!("Executing IfBlock {}", self.node_id());

        if !self.clauses.is_empty() {
            let started = Timestamp::now();

            // Explicitly re-set all clauses to inactive so it is possible to shortcut
            // evaluation by breaking on the first truthy clause
            for clause in self.clauses.iter_mut() {
                clause.is_active = Some(false);
            }

            // Iterate over clauses breaking on the first that is active
            for clause in self.clauses.iter_mut() {
                clause.execute(executor).await;

                if clause.is_active.unwrap_or_default() {
                    break;
                }
            }

            let ended = Timestamp::now();

            self.options.execution_status = Some(ExecutionStatus::Succeeded);
            self.options.execution_required = execution_required(&self.options.execution_status);
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

impl Executable for IfBlockClause {
    #[tracing::instrument(skip_all)]
    async fn execute<'lt>(&mut self, executor: &mut Executor<'lt>) -> WalkControl {
        tracing::trace!("Executing IfBlockClause {}", self.node_id());

        let mut messages = Vec::new();
        let started = Timestamp::now();

        let code = self.code.trim();
        if !code.is_empty() {
            // Evaluate code in kernels
            let (output, mut code_messages) = executor
                .kernels
                .evaluate(code, self.programming_language.as_deref())
                .await
                .unwrap_or_else(|error| {
                    (
                        Node::Null(Null),
                        vec![error_to_message("While evaluating clause", error)],
                    )
                });
            messages.append(&mut code_messages);

            // Determine truthy-ness of the code's output value
            let truthy = match &output {
                Node::Null(..) => false,
                Node::Boolean(bool) => *bool,
                Node::Integer(int) => *int > 0,
                Node::UnsignedInteger(uint) => *uint > 0,
                Node::Number(number) => *number > 0.,
                Node::String(string) => !string.is_empty(),
                Node::Array(array) => !array.is_empty(),
                Node::Object(object) => !object.is_empty(),
                _ => true,
            };

            // Execute nodes in content if truthy
            if truthy {
                if let Err(error) = self.content.walk_async(executor).await {
                    messages.push(error_to_message("While executing content", error))
                };
            }

            self.is_active = truthy.then_some(true).or(Some(false));
        } else {
            // If code is empty then this is an `else` clause so will always
            // be active (if the `IfBlock` got this far in its execution)
            if let Err(error) = self.content.walk_async(executor).await {
                messages.push(error_to_message("While executing content", error))
            };

            self.is_active = Some(true);
        }

        let ended = Timestamp::now();

        self.options.execution_status = execution_status(&messages);
        self.options.execution_required = execution_required(&self.options.execution_status);
        self.options.execution_messages = execution_messages(messages);
        self.options.execution_duration = execution_duration(&started, &ended);
        self.options.execution_ended = Some(ended);
        self.options.execution_count.get_or_insert(0).add_assign(1);

        WalkControl::Break
    }
}
