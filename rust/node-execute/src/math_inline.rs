use schema::{CompilationMessage, MathInline};

use crate::prelude::*;

impl Executable for MathInline {
    #[tracing::instrument(skip_all)]
    async fn compile(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();

        let compilation_digest = parsers::parse(
            &self.code,
            self.math_language.as_deref().unwrap_or_default(),
        )
        .compilation_digest;

        if Some(&compilation_digest) == self.options.compilation_digest.as_ref() {
            tracing::trace!("Skipping compiling MathInline {node_id}");

            return WalkControl::Break;
        }

        tracing::trace!("Compiling MathInline {node_id}");

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
                    .evaluate(&self.code, Some(&lang))
                    .await
                    .map_or_else(
                        |error| (None, vec![error_to_compilation_message(error)]),
                        |(node, messages)| {
                            let Node::String(mathml) = node else {
                                return (
                                    None,
                                    vec![CompilationMessage::new(
                                        MessageLevel::Error,
                                        "Expected a string".to_string(),
                                    )],
                                );
                            };

                            let messages = messages
                                .into_iter()
                                .map(|message| CompilationMessage {
                                    level: message.level,
                                    message: message.message,
                                    error_type: message.error_type,
                                    ..Default::default()
                                })
                                .collect();

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
        tracing::trace!("Preparing MathInline {}", self.node_id());

        // Add inline math node to document context
        executor.document_context.math_inlines.push((&*self).into());

        // Break walk because no properties need to be prepared
        WalkControl::Break
    }

    #[tracing::instrument(skip_all)]
    async fn execute(&mut self, executor: &mut Executor) -> WalkControl {
        tracing::trace!("Executing MathInline {}", self.node_id());

        // Step over the inline math node
        executor.document_context.math_inlines.step();

        // Break walk because no properties need to be executed
        WalkControl::Break
    }
}
