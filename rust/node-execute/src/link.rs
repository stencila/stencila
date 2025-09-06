use stencila_codecs::Format;
use stencila_linters::LintingOptions;
use stencila_schema::{CompilationMessage, Inline, LabelType, Link, NodeType, shortcuts::t};

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

        // Update the content of the link if it has an internal target, the
        // content is empty, or ends with zero-width-space character
        // indicating it is generated. This avoids overwriting link content that
        // comes from places like JATS (e.g. "7g" when link is to Figure 7)
        const ZERO_WIDTH_SPACE: &str = "\u{200B}";
        let is_generated_or_empty = self
            .content
            .last()
            .map(|last| match last {
                Inline::Text(last) => last.value.as_str() == ZERO_WIDTH_SPACE,
                _ => false,
            })
            .unwrap_or(true);
        if let Some(target) = self.target.strip_prefix("#")
            && is_generated_or_empty
        {
            if let Some((label_type, label)) = executor.labels.get(target) {
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
                let content = vec![t(content), t(ZERO_WIDTH_SPACE)];

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
