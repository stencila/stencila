use schema::{Citation, CompilationMessage, NodeType, replicate};

use crate::prelude::*;

impl Executable for Citation {
    async fn compile(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id: schema::NodeId = self.node_id();
        tracing::trace!("Compiling Citation {node_id}");

        if let Some(reference) = &self.options.cites {
            // The citations reference may not be in the bibliography so add it there
            // Note that we allow for each reference to be targeted using either
            // custom id or DOI as in the Article::compile method
            if let Some(id) = &reference.id {
                if !executor.bibliography.contains_key(id) {
                    executor.bibliography.insert(id.into(), reference.clone());
                }
            }
            if let Some(doi) = &reference.doi {
                if !executor.bibliography.contains_key(doi) {
                    executor.bibliography.insert(doi.into(), reference.clone());
                }
            }
        }

        WalkControl::Continue
    }

    async fn link(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::trace!("Linking Citation {node_id}");

        if let Some(reference) = executor.bibliography.get(self.target.trim()) {
            // If the reference is matched in targets and current reference is none or not equal,
            // then replicate the reference (do NOT clone to avoid duplicated id).
            if self.options.cites.is_none() || Some(reference) != self.options.cites.as_ref() {
                if let Ok(reference) = replicate(reference) {
                    self.options.cites = Some(reference.clone());
                    executor.patch(&node_id, [set(NodeProperty::Cites, reference.clone())]);
                }
            }

            if self.options.compilation_messages.is_some() {
                self.options.compilation_messages = None;
                executor.patch(&node_id, [none(NodeProperty::CompilationMessages)]);
            }
        } else if self.options.cites.is_none() {
            // Only create a compilation message if has no reference
            let messages = vec![CompilationMessage {
                level: MessageLevel::Error,
                message: format!("Unable to resolve citation target `{}`", self.target),
                ..Default::default()
            }];
            self.options.compilation_messages = Some(messages.clone());
            executor.patch(&node_id, [set(NodeProperty::CompilationMessages, messages)]);
        }

        if let Some(reference) = &self.options.cites {
            if let Some(id) = reference.doi.as_ref().or(reference.id.as_ref()) {
                // Record the reference as cited if not already and if not in a chat
                if !executor.references.contains(id)
                    && !executor.walk_ancestors.iter().any(|node_type| {
                        matches!(
                            node_type,
                            NodeType::Chat | NodeType::PromptBlock | NodeType::Excerpt
                        )
                    })
                {
                    executor.references.insert(id.clone());
                }
            }
        }

        WalkControl::Continue
    }
}
