use schema::Paragraph;

use crate::prelude::*;

impl Executable for Paragraph {
    #[tracing::instrument(skip_all)]
    async fn prepare(&mut self, executor: &mut Executor) -> WalkControl {
        tracing::trace!("Preparing Paragraph {}", self.node_id());

        // Add paragraph to document context
        executor.document_context.paragraphs.push((&*self).into());

        // Continue walk over content
        WalkControl::Continue
    }

    #[tracing::instrument(skip_all)]
    async fn execute(&mut self, executor: &mut Executor) -> WalkControl {
        tracing::trace!("Executing Paragraph {}", self.node_id());

        // Enter the paragraph context
        executor.document_context.paragraphs.enter();

        // Walk over content in case any is executable
        if let Err(error) = self.content.walk_async(executor).await {
            tracing::error!("While executing heading `content`: {error}")
        }

        // Exit the paragraph context
        executor.document_context.paragraphs.exit();

        // Break walk because content executed above
        WalkControl::Break
    }
}
