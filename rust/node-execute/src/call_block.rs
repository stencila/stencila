use schema::CallBlock;

use crate::prelude::*;

impl Executable for CallBlock {
    #[tracing::instrument(skip_all)]
    async fn execute(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();

        tracing::debug!("Executing CallBlock {node_id}");

        executor.patch(
            &node_id,
            [set(
                NodeProperty::ExecutionMessages,
                vec![ExecutionMessage::new(
                    MessageLevel::Warning,
                    "Execution of call blocks is not yet implemented".to_string(),
                )],
            )],
        );

        WalkControl::Break
    }
}
