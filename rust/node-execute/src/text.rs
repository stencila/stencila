use codecs::Format;
use schema::{NodeType, Text};
use stencila_linters::LintingOptions;

use crate::{CompileOptions, prelude::*};

impl Executable for Text {
    #[tracing::instrument(skip_all)]
    async fn compile(&mut self, executor: &mut Executor) -> WalkControl {
        let CompileOptions {
            should_lint,
            should_format,
            should_fix,
        } = executor.compile_options.clone().unwrap_or_default();

        if !should_lint {
            // Early return if not linting
            // Break walk because no other properties need to be visited
            return WalkControl::Break
        }

        let node_id = self.node_id();

        // Lint the text
        let outputs = match stencila_linters::lint(
            &self.value,
            None,
            LintingOptions {
                node_type: Some(NodeType::Text),
                format: Some(Format::Text),
                should_format,
                should_fix,
                ..Default::default()
            },
        )
        .await
        {
            Ok(outputs) => outputs,
            Err(error) => {
                tracing::error!("Error linting text: {error}");
                return WalkControl::Break;
            }
        };

        // Collect any messages and linted text from messages
        let mut new_messages = Vec::new();
        let mut new_text = None;
        for output in outputs {
            if let Some(messages) = output.messages {
                new_messages.extend(messages);
            }
            if new_text.is_none() && output.content.is_some() {
                new_text = output.content;
            }
        }

        // Only create a patch isf necessary
        let mut ops = Vec::new();
        if let Some(text) = new_text {
            ops.push(set(NodeProperty::Value, text));
        }

        if new_messages.is_empty() {
            if self.compilation_messages.is_some() {
                ops.push(none(NodeProperty::CompilationMessages));
            }
        } else {
            ops.push(set(NodeProperty::CompilationMessages, new_messages));
        }

        if !ops.is_empty() {
            executor.patch(&node_id, ops);
        }

        // Break walk because no other properties need to be visited
        WalkControl::Break
    }
}
