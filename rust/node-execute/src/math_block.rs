use schema::MathBlock;

use crate::prelude::*;

impl Executable for MathBlock {
    #[tracing::instrument(skip_all)]
    async fn pending(&mut self, _executor: &mut Executor) -> WalkControl {
        // Because `MathBlock`s do not have an execution status property
        // at present, there is nothing to be done here
        WalkControl::Break
    }

    #[tracing::instrument(skip_all)]
    async fn execute(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::trace!("Executing MathBlock {node_id}");

        let code = self.code.trim();
        if !code.is_empty() {
            let (mut outputs, messages) = executor
                .kernels()
                .await
                .execute(code, self.math_language.as_deref())
                .await
                .unwrap_or_else(|error| {
                    (
                        Vec::new(),
                        vec![error_to_message("While compiling math", error)],
                    )
                });

            let mathml = (!outputs.is_empty()).then(|| outputs.swap_remove(0));
            let messages = (!messages.is_empty()).then_some(messages);

            executor.replace_properties(
                &node_id,
                [
                    (Property::MathMl, mathml.into()),
                    (Property::CompilationMessages, messages.into()),
                ],
            );
        } else {
            executor.replace_properties(
                &node_id,
                [
                    (Property::MathMl, Value::None),
                    (Property::CompilationMessages, Value::None),
                ],
            );
        };

        WalkControl::Break
    }

    #[tracing::instrument(skip_all)]
    async fn interrupt(&mut self, _executor: &mut Executor) -> WalkControl {
        // Because `MathBlock`s can not be interrupted there is nothing to be done here
        WalkControl::Break
    }
}
