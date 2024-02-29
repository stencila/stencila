use schema::Article;

use crate::prelude::*;

impl Executable for Article {
    #[tracing::instrument(skip_all)]
    async fn execute<'lt>(&mut self, executor: &mut Executor<'lt>) -> WalkControl {
        tracing::trace!("Executing Article {}", self.node_id());

        let mut messages = Vec::new();
        let started = Timestamp::now();

        if let Err(error) = self.content.walk_async(executor).await {
            messages.push(error_to_message("While executing content", error));
        }

        let ended = Timestamp::now();

        // TODO: set execution_status based on the execution status of
        // child executable nodes

        // TODO: set execution_required based on execution status

        self.options.execution_status = execution_status(&messages);
        self.options.execution_required = execution_required(&self.options.execution_status);
        self.options.execution_messages = execution_messages(messages);
        self.options.execution_duration = execution_duration(&started, &ended);
        self.options.execution_ended = Some(ended);
        self.options.execution_count.get_or_insert(0).add_assign(1);

        WalkControl::Break
    }
}
