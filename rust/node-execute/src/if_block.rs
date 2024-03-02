use schema::{IfBlock, IfBlockClause};

use crate::{interrupt_impl, pending_impl, prelude::*};

impl Executable for IfBlock {
    #[tracing::instrument(skip_all)]
    async fn pending(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::debug!("Pending IfBlock {node_id}");

        pending_impl!(executor, &node_id);

        // Break so that clauses (and `content` in clauses) are not made pending
        WalkControl::Break
    }

    #[tracing::instrument(skip_all)]
    async fn execute(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::trace!("Executing IfBlock {node_id}");

        executor.replace_properties(
            &node_id,
            [
                (Property::ExecutionStatus, ExecutionStatus::Running.into()),
                (Property::ExecutionMessages, Value::None),
            ],
        );

        if !self.clauses.is_empty() {
            let started = Timestamp::now();

            // Explicitly re-set all clauses to inactive so it is possible to shortcut
            // evaluation by breaking on the first truthy clause
            for clause in self.clauses.iter_mut() {
                clause.is_active = Some(false);
            }

            // Iterate over clauses breaking on the first that is active
            // and determine execution status based on highest status of executed clauses
            let mut status = ExecutionStatus::Succeeded;
            let last_index = self.clauses.len() - 1;
            for (index, clause) in self.clauses.iter_mut().enumerate() {
                executor.is_last = index == last_index;
                clause.execute(executor).await;

                if let Some(clause_status) = &clause.options.execution_status {
                    if clause_status > &status {
                        status = clause_status.clone();
                    }
                }

                if clause.is_active.unwrap_or_default() {
                    break;
                }
            }

            let ended = Timestamp::now();

            let required = execution_required(&status);
            let duration = execution_duration(&started, &ended);
            let count = self.options.execution_count.unwrap_or_default() + 1;

            executor.replace_properties(
                &node_id,
                [
                    (Property::ExecutionStatus, status.into()),
                    (Property::ExecutionRequired, required.into()),
                    (Property::ExecutionDuration, duration.into()),
                    (Property::ExecutionEnded, ended.into()),
                    (Property::ExecutionCount, count.into()),
                ],
            );
        } else {
            executor.replace_properties(
                &node_id,
                [
                    (Property::ExecutionStatus, ExecutionStatus::Empty.into()),
                    (Property::ExecutionRequired, ExecutionRequired::No.into()),
                    (Property::ExecutionDuration, Value::None),
                    (Property::ExecutionEnded, Value::None),
                ],
            );
        }

        WalkControl::Break
    }

    #[tracing::instrument(skip_all)]
    async fn interrupt(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::debug!("Interrupting IfBlock {node_id}");

        interrupt_impl!(self, executor, &node_id);

        // Continue to interrupt `clauses` and any executable nodes within them
        WalkControl::Continue
    }
}

impl Executable for IfBlockClause {
    #[tracing::instrument(skip_all)]
    async fn pending(&mut self, _executor: &mut Executor) -> WalkControl {
        // No change to execution status because not every clause will be
        // executed (breaks on first truthy) so setting to `Pending` here
        // could never be overwritten.
        WalkControl::Break
    }

    #[tracing::instrument(skip_all)]
    async fn execute(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::trace!("Executing IfBlockClause {node_id}");

        executor.replace_properties(
            &node_id,
            [
                (Property::ExecutionStatus, ExecutionStatus::Running.into()),
                (Property::ExecutionMessages, Value::None),
            ],
        );

        let mut messages = Vec::new();
        let started = Timestamp::now();

        let code = self.code.trim();
        let (is_active, mut status) = if !code.is_empty() {
            // Evaluate code in kernels
            let (output, mut code_messages) = executor
                .kernels
                .write()
                .await
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

            (truthy, ExecutionStatus::Running)
        } else if code.is_empty() && executor.is_last {
            // If code is empty and this is the last clause then this is an `else` clause so will always
            // be active (if the `IfBlock` got this far in its execution)
            if let Err(error) = self.content.walk_async(executor).await {
                messages.push(error_to_message("While executing content", error))
            };

            (true, ExecutionStatus::Running)
        } else {
            // Just skip if empty code and not last
            (false, ExecutionStatus::Empty)
        };

        let messages = (!messages.is_empty()).then_some(messages);

        let ended = Timestamp::now();

        if status != ExecutionStatus::Skipped {
            status = execution_status(&messages)
        }
        let required = execution_required(&status);
        let duration = execution_duration(&started, &ended);
        let count = self.options.execution_count.unwrap_or_default() + 1;

        // Update `is_active` on `self` so that parent `IfBlock` can break
        // on first active clause
        self.is_active = Some(is_active);

        // Update `execution_status` on `self` so that parent `IfBlock`
        // can update its status based on clauses
        self.options.execution_status = Some(status.clone());

        executor.replace_properties(
            &node_id,
            [
                (Property::IsActive, is_active.into()),
                (Property::ExecutionStatus, status.into()),
                (Property::ExecutionRequired, required.into()),
                (Property::ExecutionMessages, messages.into()),
                (Property::ExecutionDuration, duration.into()),
                (Property::ExecutionEnded, ended.into()),
                (Property::ExecutionCount, count.into()),
            ],
        );

        WalkControl::Break
    }

    #[tracing::instrument(skip_all)]
    async fn interrupt(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::debug!("Interrupting IfBlockClause {node_id}");

        interrupt_impl!(self, executor, &node_id);

        // Continue to interrupt executable nodes in `content`
        WalkControl::Continue
    }
}
