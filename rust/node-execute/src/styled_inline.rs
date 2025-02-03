use schema::{CompilationMessage, StyledInline};

use crate::prelude::*;

impl Executable for StyledInline {
    #[tracing::instrument(skip_all)]
    async fn compile(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();

        // Parse the code to determine if it or the language has changed since last time
        let info = parsers::parse(
            &self.code,
            &self.style_language,
            &self.options.compilation_digest,
        );

        // Return early if no change
        if info.changed.no() {
            tracing::trace!("Skipping compiling StyledInline {node_id}");

            // Continue to compile child nodes in content
            return WalkControl::Continue;
        }

        tracing::trace!("Compiling StyledInline {node_id}");

        if !self.code.trim().is_empty() {
            let lang = self.style_language.as_deref().or(Some("style"));

            let (result, messages) = executor
                .kernels()
                .await
                .execute(&self.code, lang)
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
            let class_list = match result.next() {
                Some(Node::String(value)) => Some(value),
                _ => None,
            };

            let messages = (!messages.is_empty()).then_some(messages);

            executor.patch(
                &node_id,
                [
                    set(NodeProperty::Css, css),
                    set(NodeProperty::ClassList, class_list),
                    set(NodeProperty::CompilationMessages, messages),
                    set(NodeProperty::CompilationDigest, info.compilation_digest),
                ],
            );
        } else {
            executor.patch(
                &node_id,
                [
                    none(NodeProperty::Css),
                    none(NodeProperty::ClassList),
                    none(NodeProperty::CompilationMessages),
                    set(NodeProperty::CompilationDigest, info.compilation_digest),
                ],
            );
        };

        WalkControl::Continue
    }

    #[tracing::instrument(skip_all)]
    async fn execute(&mut self, executor: &mut Executor) -> WalkControl {
        // Re-compile in case required variables were not available on compile
        // TODO: a more refined approached based on any interpolated dependencies is needed
        self.options.compilation_digest = None;
        self.compile(executor).await
    }
}
