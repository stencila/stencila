use schema::Heading;

use crate::prelude::*;

impl Executable for Heading {
    #[tracing::instrument(skip_all)]
    async fn prepare(&mut self, executor: &mut Executor) -> WalkControl {
        tracing::trace!("Preparing Heading {}", self.node_id());

        // Add heading to document context
        executor.document_context.headings.push((&*self).into());

        WalkControl::Continue
    }

    #[tracing::instrument(skip_all)]
    async fn execute(&mut self, executor: &mut Executor) -> WalkControl {
        tracing::trace!("Executing Heading {}", self.node_id());

        // Move the context cursor for headings forward
        executor.document_context.headings.forward();

        WalkControl::Continue
    }
}
