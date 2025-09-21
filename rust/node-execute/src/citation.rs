use stencila_schema::{Citation, CitationGroup, CompilationMessage, NodeId, replicate};

use crate::prelude::*;

impl Executable for Citation {
    async fn compile(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id: NodeId = self.node_id();
        tracing::trace!("Compiling Citation {node_id}");

        if let Some(reference) = executor.bibliography.get(self.target.trim()) {
            // If the citation's target is in the bibliography and the currently
            // cited reference is None or not equal, then replicate the
            // reference (do NOT clone to avoid duplicated id)
            if (self.options.cites.is_none() || Some(reference) != self.options.cites.as_ref())
                && let Ok(mut reference) = replicate(reference)
            {
                // Remove the rendered content of the cited reference to avoid
                // unnecessary memory usage
                reference.options.content = None;

                self.options.cites = Some(reference.clone());
                executor.patch(&node_id, [set(NodeProperty::Cites, reference)]);
            }

            // If the citation has any compilation messages, remove them since
            // they are now outdated (because the citation has been successfully linked).
            if self.options.compilation_messages.is_some() {
                self.options.compilation_messages = None;
                executor.patch(&node_id, [none(NodeProperty::CompilationMessages)]);
            }
        } else if self.options.cites.is_none() {
            // The citation's target could not be found in the executor's bibliography and it does not
            // have its own cited reference, so create a compilation message
            let messages = vec![CompilationMessage {
                level: MessageLevel::Error,
                error_type: Some("Target Unresolved".into()),
                message: format!("Unable to resolve citation target `{}`", self.target),
                ..Default::default()
            }];
            self.options.compilation_messages = Some(messages.clone());
            executor.patch(&node_id, [set(NodeProperty::CompilationMessages, messages)]);
        }

        // If the citation has a cited reference (either target was found in bibliography, or it
        // has an existing `cites`) then record it in the executor's citations so that content
        // can be rendered for the citation and all the cited references
        if self.options.cites.is_some() {
            let citation_group = CitationGroup::new(vec![self.clone()]);
            executor.citations.insert(node_id, (citation_group, None));
        }

        // Break walk because no need to walk over `content` (or other properties)
        WalkControl::Break
    }

    async fn link(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::trace!("Linking Citation {node_id}");

        // (Re)populate the citation's content
        // Note: this is only called for standalone citations, not those that are part
        // of a citation group (the latter is not necessary)
        if let Some(content) = executor
            .citations
            .get_mut(&node_id)
            .and_then(|(.., content)| content.take())
        {
            self.options.content = Some(content.clone());
            executor.patch(&node_id, [set(NodeProperty::Content, content)]);
        }

        // Break walk because no need to walk over `content` (or other properties)
        WalkControl::Break
    }
}
