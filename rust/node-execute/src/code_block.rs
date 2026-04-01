use stencila_codecs::{DecodeOptions, Format};
use stencila_schema::{Article, CodeBlock, CompilationMessage};

use crate::prelude::*;

impl Executable for CodeBlock {
    #[tracing::instrument(skip_all)]
    async fn compile(&mut self, executor: &mut Executor) -> WalkControl {
        // Only compile if this is a demo code block
        if self.is_demo != Some(true) {
            return WalkControl::Break;
        }

        let node_id = self.node_id();

        // Determine the format from programming_language
        let format = self
            .programming_language
            .as_ref()
            .map(|lang| Format::from_name(lang))
            .unwrap_or(Format::Unknown);

        // Parse code to create compilation digest and check if changed
        let info = stencila_parsers::parse(
            &self.code,
            &self.programming_language,
            &self.options.compilation_digest,
        );

        // Return early if no change, but continue walk to compile existing content
        if info.changed.no() {
            tracing::trace!("Skipping compiling CodeBlock demo {node_id}");
            return WalkControl::Continue;
        }

        tracing::trace!("Compiling CodeBlock demo {node_id}");

        let mut messages = Vec::new();

        if !self.code.trim().is_empty() {
            // Decode the code as document content
            match stencila_codecs::from_str(
                &self.code,
                Some(DecodeOptions {
                    format: Some(format),
                    ..Default::default()
                }),
            )
            .await
            {
                Ok(Node::Article(Article { content, .. })) => {
                    self.options.content = Some(content.clone());
                    executor.patch(
                        &node_id,
                        [
                            // Use `none` and `append` to retain node ids
                            none(NodeProperty::Content),
                            append(NodeProperty::Content, content),
                        ],
                    );
                }
                Ok(_) => {
                    messages.push(CompilationMessage::new(
                        MessageLevel::Error,
                        "Expected content to be decoded to an article".to_string(),
                    ));
                    self.options.content = None;
                    executor.patch(&node_id, [none(NodeProperty::Content)]);
                }
                Err(error) => {
                    messages.push(error_to_compilation_message(error));
                    self.options.content = None;
                    executor.patch(&node_id, [none(NodeProperty::Content)]);
                }
            }
        } else {
            self.options.content = None;
            executor.patch(&node_id, [none(NodeProperty::Content)]);
        }

        // Walk into content to compile nested nodes (figures, math, etc.)
        if let Err(error) = self.options.content.walk_async(executor).await {
            messages.push(error_to_compilation_message(error));
        }

        let messages = (!messages.is_empty()).then_some(messages);
        self.options.compilation_messages = messages.clone();

        executor.patch(
            &node_id,
            [
                set(NodeProperty::CompilationMessages, messages),
                set(NodeProperty::CompilationDigest, info.compilation_digest),
            ],
        );

        // Break because content already walked above
        WalkControl::Break
    }
}
