use codec_markdown::decode_frontmatter;
use schema::{NodeType, Prompt};

use crate::prelude::*;

impl Executable for Prompt {
    #[tracing::instrument(skip_all)]
    async fn compile(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::trace!("Compiling Prompt {node_id}");

        let mut messages = Vec::new();

        // Check frontmatter for syntactic and semantic errors
        if let Some(yaml) = self.frontmatter.as_deref() {
            let (.., mut fm_messages) = decode_frontmatter(yaml, Some(NodeType::Article));
            messages.append(&mut fm_messages);
        }

        if messages.is_empty() {
            self.options.compilation_messages = None;
            executor.patch(&node_id, [none(NodeProperty::CompilationMessages)]);
        } else {
            self.options.compilation_messages = Some(messages.clone());
            executor.patch(&node_id, [set(NodeProperty::CompilationMessages, messages)]);
        }

        WalkControl::Continue
    }
}
