use schema::Section;

use crate::prelude::*;

impl Executable for Section {
    #[tracing::instrument(skip_all)]
    async fn prepare(&mut self, executor: &mut Executor) -> WalkControl {
        tracing::trace!("Preparing Section {}", self.node_id());

        // Add section to document context
        executor.document_context.sections.push((&*self).into());

        // Continue walk over content
        WalkControl::Continue
    }

    #[tracing::instrument(skip_all)]
    async fn execute(&mut self, executor: &mut Executor) -> WalkControl {
        tracing::trace!("Executing Section {}", self.node_id());

        // Enter the section context
        executor.document_context.sections.enter();

        // Walk over content in case any is executable
        if let Err(error) = self.content.walk_async(executor).await {
            tracing::error!("While executing section `content`: {error}")
        }

        // Exit the section context
        executor.document_context.sections.exit();

        // Break walk because content executed above
        WalkControl::Break
    }
}
