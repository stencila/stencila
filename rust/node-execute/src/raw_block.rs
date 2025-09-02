use stencila_codecs::Format;
use stencila_schema::{CompilationMessage, RawBlock};

use crate::prelude::*;

impl Executable for RawBlock {
    #[tracing::instrument(skip_all)]
    async fn compile(&mut self, executor: &mut Executor) -> WalkControl {
        // Return early if not a format that requires compiling
        let format = Format::from_name(&self.format);
        if !matches!(format, Format::Html | Format::Css) {
            // Break walk because no properties need compiling
            return WalkControl::Break;
        }

        let node_id = self.node_id();

        // Parse the code to determine if it or the format has changed since last time
        let info = stencila_parsers::parse(
            &self.content,
            &Some(self.format.clone()),
            &self.compilation_digest,
        );

        // Return early if no change
        if info.changed.no() {
            tracing::trace!("Skipping compiling RawBlock {node_id}");

            // Break walk because no properties need compiling
            return WalkControl::Break;
        }

        tracing::trace!("Compiling RawBlock {node_id}");

        if !self.content.trim().is_empty() {
            let (result, messages) = executor
                .kernels()
                .await
                .execute(self.content.trim(), Some(format.to_string().as_str()))
                .await
                .map_or_else(
                    |error| (None, vec![error_to_compilation_message(error)]),
                    |(outputs, messages, ..)| {
                        let messages = messages.into_iter().map(CompilationMessage::from).collect();
                        (Some(outputs), messages)
                    },
                );

            let mut result = result.into_iter().flatten();
            let css = match result.next() {
                Some(Node::String(value)) => Some(value),
                _ => None,
            };

            let messages = (!messages.is_empty()).then_some(messages);

            executor.patch(
                &node_id,
                [
                    set(NodeProperty::Css, css),
                    set(NodeProperty::CompilationMessages, messages),
                    set(NodeProperty::CompilationDigest, info.compilation_digest),
                ],
            );
        } else {
            executor.patch(
                &node_id,
                [
                    none(NodeProperty::Css),
                    none(NodeProperty::CompilationMessages),
                    set(NodeProperty::CompilationDigest, info.compilation_digest),
                ],
            );
        };

        // Break walk because no other properties need compiling
        return WalkControl::Break;
    }
}
