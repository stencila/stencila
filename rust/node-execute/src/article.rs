use schema::Article;

use crate::{interrupt_impl, pending_impl, prelude::*};

impl Executable for Article {
    #[tracing::instrument(skip_all)]
    async fn pending(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::debug!("Pending Article {node_id}");

        pending_impl!(executor, &node_id);

        // Continue to mark executable nodes in `content` as pending
        WalkControl::Continue
    }

    #[tracing::instrument(skip_all)]
    async fn execute(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::trace!("Executing Article {node_id}");

        executor.replace_properties(
            &node_id,
            [
                (Property::ExecutionStatus, ExecutionStatus::Running.into()),
                (Property::ExecutionMessages, Value::None),
            ],
        );

        let started = Timestamp::now();

        let messages = if let Err(error) = self.content.walk_async(executor).await {
            Some(vec![error_to_message("While executing content", error)])
        } else {
            None
        };

        let ended = Timestamp::now();

        // TODO: set status based on the execution status of
        // child executable nodes

        let status = execution_status(&messages);
        let required = execution_required(&status);
        let duration = execution_duration(&started, &ended);
        let count = self.options.execution_count.unwrap_or_default() + 1;

        executor.replace_properties(
            &node_id,
            [
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
        tracing::debug!("Interrupting Article {node_id}");

        interrupt_impl!(self, executor, &node_id);

        // Continue to interrupt executable nodes in `content`
        WalkControl::Continue
    }
}
