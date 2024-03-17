use schema::MathInline;

use crate::prelude::*;

impl Executable for MathInline {
    #[tracing::instrument(skip_all)]
    async fn pending(&mut self, _executor: &mut Executor) -> WalkControl {
        // Because `MathInline`s do not have an execution status property
        // at present, there is nothing to be done here
        WalkControl::Break
    }

    #[tracing::instrument(skip_all)]
    async fn execute(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::trace!("Executing MathInline {node_id}");

        let code = self.code.trim();
        if !code.is_empty() {
            let (mathml, messages) = executor
                .kernels()
                .await
                .evaluate(code, self.math_language.as_deref())
                .await
                .unwrap_or_else(|error| {
                    (
                        Node::String(String::new()),
                        vec![error_to_message("While compiling math", error)],
                    )
                });

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
        // Because `MathInline`s can not be interrupted there is nothing to be done here
        WalkControl::Break
    }
}
