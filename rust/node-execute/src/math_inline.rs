use stencila_schema::{CompilationMessage, MathInline};

use crate::prelude::*;

impl Executable for MathInline {
    #[tracing::instrument(skip_all)]
    async fn compile(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();

        // Parse the code to determine if it or the language has changed since last time
        let info = stencila_parsers::parse(
            &self.code,
            &self.math_language,
            &self.options.compilation_digest,
        );

        // Return early if no change
        if info.changed.no() {
            tracing::trace!("Skipping compiling MathInline {node_id}");

            return WalkControl::Break;
        }

        tracing::trace!("Compiling MathInline {node_id}");

        if self.options.compilation_digest.is_none() && self.options.mathml.is_some() {
            // If this has not been compiled but still has MathML then it must have
            // been generated externally (e.g. from LaTeX by LaTeXML in arxiv HML) so do not touch it
        } else if !self.code.trim().is_empty() {
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
                        |(node, messages, ..)| {
                            let mathml = match node {
                                Node::String(mathml) => Some(mathml),
                                _ => None,
                            };

                            let messages =
                                messages.into_iter().map(CompilationMessage::from).collect();

                            (mathml, messages)
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
                    set(NodeProperty::CompilationDigest, info.compilation_digest),
                ],
            );
        } else {
            executor.patch(
                &node_id,
                [
                    none(NodeProperty::Mathml),
                    none(NodeProperty::CompilationMessages),
                    set(NodeProperty::CompilationDigest, info.compilation_digest),
                ],
            );
        };

        // Break walk because no other properties need to be compiled
        WalkControl::Break
    }
}
