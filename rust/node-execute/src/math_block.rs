use schema::MathBlock;

use crate::prelude::*;

impl Executable for MathBlock {
    #[tracing::instrument(skip_all)]
    async fn execute(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::trace!("Executing MathBlock {node_id}");

        let code = self.code.trim();
        if !code.is_empty() {
            let lang = self
                .math_language
                .as_ref()
                .map_or("tex".to_string(), |lang| lang.to_lowercase());

            let (mathml, messages) = if lang == "mathml" {
                (Some(Node::String(code.to_string())), None)
            } else {
                let (mut outputs, messages) = executor
                    .kernels()
                    .await
                    .execute(code, Some(&lang))
                    .await
                    .unwrap_or_else(|error| {
                        (
                            Vec::new(),
                            vec![error_to_message("While compiling math", error)],
                        )
                    });

                let mathml = (!outputs.is_empty()).then(|| outputs.swap_remove(0));
                let messages = (!messages.is_empty()).then_some(messages);

                (mathml, messages)
            };

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
}
