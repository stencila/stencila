use stencila_schema::CitationGroup;

use crate::prelude::*;

impl Executable for CitationGroup {
    async fn compile(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::trace!("Compiling CitationGroup {node_id}");

        // Compile each citation in `items`
        let mut has_cites = false;
        for item in &mut self.items {
            item.compile(executor).await;

            if !has_cites && item.options.cites.is_some() {
                has_cites = true;
            }
        }

        // If any of the the citations has a cited reference (either target was
        // found in bibliography, or it has an existing `cites`) then record
        // this group in executor's citations so that content can be rendered
        // for the citations and all the cited references
        if has_cites {
            executor.citations.insert(node_id, (self.clone(), None));
        }

        // Break walk because `items` have already been visited above
        WalkControl::Break
    }

    async fn link(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::trace!("Linking CitationGroup {node_id}");

        // (Re)populate the citation groups's content if it is a standalone
        // citation (i.e. not part of a CitationGroup)
        if let Some(content) = executor
            .citations
            .get_mut(&node_id)
            .and_then(|(.., content)| content.take())
        {
            self.content = Some(content.clone());
            executor.patch(&node_id, [set(NodeProperty::Content, content)]);
        }

        // Break walk because the `items` within a citation group do not need to be
        // linked (their content should be None)
        WalkControl::Break
    }
}
