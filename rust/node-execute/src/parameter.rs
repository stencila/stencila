use schema::Parameter;

use crate::prelude::*;

impl Executable for Parameter {
    #[tracing::instrument(skip_all)]
    async fn execute(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();

        tracing::debug!("Executing Parameter {node_id}");

        executor.patch(
            &node_id,
            [set(
                NodeProperty::ExecutionMessages,
                vec![ExecutionMessage::new(
                    MessageLevel::Warning,
                    "Execution of parameters is not yet implemented".to_string(),
                )],
            )],
        );

        WalkControl::Break
    }
}
