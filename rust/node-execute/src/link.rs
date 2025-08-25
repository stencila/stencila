use codecs::Format;
use schema::{CompilationMessage, LabelType, Link, NodeType, shortcuts::t};
use stencila_linters::LintingOptions;

use crate::{CompileOptions, prelude::*};

impl Executable for Link {
    #[tracing::instrument(skip_all)]
    async fn compile(&mut self, executor: &mut Executor) -> WalkControl {
        let CompileOptions {
            should_lint,
            should_format,
            should_fix,
        } = executor.compile_options.clone().unwrap_or_default();

        if !should_lint {
            // Early return if not linting
            // Continue walk to visit other properties
            return WalkControl::Continue;
        }

        let node_id = self.node_id();

        // Create content string for linting by serializing the target URL
        let content = self.target.clone();

        // Lint the link target
        let output = match stencila_linters::lint(
            &content,
            None,
            LintingOptions {
                node_type: Some(NodeType::Link),
                format: Some(Format::Text),
                should_format,
                should_fix,
                ..Default::default()
            },
        )
        .await
        {
            Ok(output) => output,
            Err(error) => {
                tracing::error!("Error linting link: {error}");
                return WalkControl::Continue;
            }
        };

        // Collect any messages from linting
        let new_messages = output.messages.unwrap_or_default();

        // Only create a patch if necessary
        let mut ops = Vec::new();

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

        // Continue walk to visit other properties
        WalkControl::Continue
    }

    async fn link(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::trace!("Linking Link {node_id}");

        // Update the content of the link if it has an internal target
        if !(self.target.starts_with("https://") || self.target.starts_with("http://")) {
            if let Some((label_type, label)) = executor.labels.get(&self.target) {
                let label_type = match label_type {
                    LabelType::TableLabel => "Table",
                    LabelType::FigureLabel => "Figure",
                    LabelType::AppendixLabel => "Appendix",
                };

                let content = if self.label_only.unwrap_or_default() {
                    label.clone()
                } else {
                    [label_type, " ", label].concat()
                };
                let content = vec![t(content)];

                self.content = content.clone();
                executor.patch(
                    &node_id,
                    [
                        clear(NodeProperty::Content),
                        append(NodeProperty::Content, content),
                        none(NodeProperty::CompilationMessages),
                    ],
                );
            } else {
                let messages = vec![CompilationMessage {
                    level: MessageLevel::Error,
                    error_type: Some("Target Unresolved".into()),
                    message: format!("Unable to resolve internal link target `{}`", self.target),
                    ..Default::default()
                }];
                self.compilation_messages = Some(messages.clone());
                executor.patch(&node_id, [set(NodeProperty::CompilationMessages, messages)]);
            }
        }

        WalkControl::Continue
    }
}
