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

        let code = self.code.trim();
        if !code.is_empty() {
            let lang = self
                .math_language
                .as_ref()
                .map_or("tex".to_string(), |lang| lang.to_lowercase());

            let (mathml, messages) = if lang == "mathml" {
                (Some(code.to_string()), None)
            } else {
                let (mathml, messages) = executor
                    .kernels()
                    .await
                    .execute(code, Some(&lang))
                    .await
                    .map_or_else(
                        |error| {
                            (
                                None,
                                vec![error_to_compilation_message(error)],
                            )
                        },
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

        WalkControl::Continue
    }
}
