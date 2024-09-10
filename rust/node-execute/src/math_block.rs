use schema::{CompilationMessage, MathBlock};

use crate::prelude::*;

impl Executable for MathBlock {
    #[tracing::instrument(skip_all)]
    async fn compile(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();

        let compilation_digest = parsers::parse(
            &self.code,
            self.math_language.as_deref().unwrap_or_default(),
        )
        .compilation_digest;

        if Some(&compilation_digest) == self.options.compilation_digest.as_ref() {
            tracing::trace!("Skipping compiling MathBlock {node_id}");

            return WalkControl::Break;
        }

        tracing::trace!("Compiling MathBlock {node_id}");

        executor.equation_count += 1;

        if self.label_automatically.unwrap_or(true) {
            let label = executor.equation_count.to_string();
            if Some(&label) != self.label.as_ref() {
                executor.patch(&node_id, [set(NodeProperty::Label, label)]);
            }
        }

        if !self.code.trim().is_empty() {
            let lang = self
                .math_language
                .as_ref()
                .map_or("tex".to_string(), |lang| lang.to_lowercase());

            let (mathml, messages) = if lang == "mathml" {
                (Some(self.code.to_string()), None)
            } else {
                let (mathml, messages) = executor
                    .kernels()
                    .await
                    .execute(&self.code, Some(&lang))
                    .await
                    .map_or_else(
                        |error| (None, vec![error_to_compilation_message(error)]),
                        |(mut outputs, messages)| {
                            let output = (!outputs.is_empty()).then(|| outputs.swap_remove(0));
                            let Some(Node::String(mathml)) = output else {
                                return (
                                    None,
                                    vec![CompilationMessage::new(
                                        MessageLevel::Error,
                                        "Expected a string".to_string(),
                                    )],
                                );
                            };

                            let messages =
                                messages.into_iter().map(CompilationMessage::from).collect();

                            (Some(mathml), messages)
                        },
                    );

                let messages = (!messages.is_empty()).then_some(messages);

                (mathml, messages)
            };

            executor.patch(
                &node_id,
                [
                    set(NodeProperty::Mathml, mathml),
                    set(NodeProperty::CompilationMessages, messages),
                    set(NodeProperty::CompilationDigest, compilation_digest),
                ],
            );
        } else {
            executor.patch(
                &node_id,
                [
                    none(NodeProperty::Mathml),
                    none(NodeProperty::CompilationMessages),
                    set(NodeProperty::CompilationDigest, compilation_digest),
                ],
            );
        };

        // Break walk because no other properties need to be compiled
        WalkControl::Break
    }

    #[tracing::instrument(skip_all)]
    async fn prepare(&mut self, executor: &mut Executor) -> WalkControl {
        tracing::trace!("Preparing MathBlock {}", self.node_id());

        // Add math block to document context
        executor.document_context.math_blocks.push((&*self).into());

        // Break walk because no properties need to be prepared
        WalkControl::Break
    }

    #[tracing::instrument(skip_all)]
    async fn execute(&mut self, executor: &mut Executor) -> WalkControl {
        tracing::trace!("Executing MathBlock {}", self.node_id());

        // Step over the math block
        executor.document_context.math_blocks.step();

        // Break walk because no properties need to be executed
        WalkControl::Break
    }
}
