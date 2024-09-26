use schema::{diff, Article, PatchSlot};

use crate::{interrupt_impl, prelude::*, HeadingInfo};

impl Executable for Article {
    #[tracing::instrument(skip_all)]
    async fn compile(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::trace!("Compiling Article {node_id}");

        // Clear the executor's headings
        executor.headings.clear();

        // Compile the `content` and `title` (could include math)
        if let Err(error) = async {
            self.title.walk_async(executor).await?;
            self.content.walk_async(executor).await
        }
        .await
        {
            tracing::error!("While compiling article: {error}")
        }

        // Ensure any trailing headings are collapsed into their parents
        HeadingInfo::collapse(1, &mut executor.headings);

        // Transform the executors heading info
        let headings = (!executor.headings.is_empty())
            .then(|| HeadingInfo::into_list(executor.headings.drain(..).collect()));

        // Diff the headings list with the current, prepend any generated diff ops
        // with the path to headings and send a patch if necessary
        match diff(&self.headings, &headings, None, None) {
            Ok(mut patch) => {
                patch.node_id = Some(node_id);
                if !patch.ops.is_empty() {
                    patch.prepend_paths(vec![PatchSlot::Property(NodeProperty::Headings)]);
                    executor.send_patch(patch);
                }
            }
            Err(error) => {
                tracing::error!("While diffing article headings: {error}")
            }
        }

        // Break walk because `content` and `title` already walked over
        WalkControl::Break
    }

    #[tracing::instrument(skip_all)]
    async fn prepare(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::trace!("Preparing Article {node_id}");

        // Add article metadata to document context
        executor.document_context.metadata = (&*self).into();

        // Set execution status
        self.options.execution_status = Some(ExecutionStatus::Pending);
        executor.patch(
            &node_id,
            [set(NodeProperty::ExecutionStatus, ExecutionStatus::Pending)],
        );

        // Continue to prepare executable nodes in `content`
        WalkControl::Continue
    }

    #[tracing::instrument(skip_all)]
    async fn execute(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::debug!("Executing Article {node_id}");

        executor.patch(
            &node_id,
            [
                set(NodeProperty::ExecutionStatus, ExecutionStatus::Running),
                none(NodeProperty::ExecutionMessages),
            ],
        );

        let started = Timestamp::now();

        let messages = if let Err(error) = self.content.walk_async(executor).await {
            Some(vec![error_to_execution_message(
                "While executing content",
                error,
            )])
        } else {
            None
        };

        let ended = Timestamp::now();

        // TODO: set status based on the execution status of
        // child executable nodes

        let status = execution_status(&messages);
        let required = execution_required_status(&status);
        let duration = execution_duration(&started, &ended);
        let count = self.options.execution_count.unwrap_or_default() + 1;

        executor.patch(
            &node_id,
            [
                set(NodeProperty::ExecutionStatus, status),
                set(NodeProperty::ExecutionRequired, required),
                set(NodeProperty::ExecutionMessages, messages),
                set(NodeProperty::ExecutionDuration, duration),
                set(NodeProperty::ExecutionEnded, ended),
                set(NodeProperty::ExecutionCount, count),
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
