use schema::Paragraph;

use crate::prelude::*;

impl Executable for Paragraph {
    #[tracing::instrument(skip_all)]
    async fn prepare(&mut self, executor: &mut Executor) -> WalkControl {
        tracing::trace!("Preparing Paragraph {}", self.node_id());

        // Add paragraph to document context
        executor.document_context.paragraphs.push((&*self).into());

        WalkControl::Continue
    }

    #[tracing::instrument(skip_all)]
    async fn execute(&mut self, executor: &mut Executor) -> WalkControl {
        tracing::trace!("Executing Paragraph {}", self.node_id());

        // Move the context cursor for paragraphs forward
        executor.document_context.paragraphs.forward();

        WalkControl::Continue
    }
}
