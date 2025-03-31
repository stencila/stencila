use schema::{Heading, NodeType};

use crate::{prelude::*, HeadingInfo};

impl Executable for Heading {
    #[tracing::instrument(skip_all)]
    async fn compile(&mut self, executor: &mut Executor) -> WalkControl {
        // If necessary, collapse previous headings into their parents
        HeadingInfo::collapse(self.level, &mut executor.headings);

        // Record this heading if appropriate
        if !executor.walk_ancestors.iter().any(|node_type| {
            matches!(
                node_type,
                NodeType::Figure | NodeType::Table | NodeType::CodeChunk
            )
        }) {
            let info = HeadingInfo {
                level: self.level,
                node_id: self.node_id(),
                content: self.content.clone(),
                children: Vec::new(),
            };
            executor.headings.push(info);
        }

        // Continue walk over content
        WalkControl::Continue
    }

    #[tracing::instrument(skip_all)]
    async fn prepare(&mut self, executor: &mut Executor) -> WalkControl {
        tracing::trace!("Preparing Heading {}", self.node_id());

        // Add heading to document context
        executor.document_context.headings.push((&*self).into());
        executor.document_context.sections.push_heading(&*self);

        // Continue walk over content
        WalkControl::Continue
    }

    #[tracing::instrument(skip_all)]
    async fn execute(&mut self, executor: &mut Executor) -> WalkControl {
        tracing::trace!("Executing Heading {}", self.node_id());

        // Enter the heading context
        executor.document_context.headings.enter();
        executor.document_context.sections.enter_heading(&*self);

        // Walk over content in case any is executable
        if let Err(error) = self.content.walk_async(executor).await {
            tracing::error!("While executing heading `content`: {error}")
        }

        // Exit the heading context. Note that we do not call exit on `sections` (as done
        // above for `enter`) because a section defined by a level 1 heading only finishes
        // at the next level one heading.
        executor.document_context.headings.exit();

        // Break walk because content executed above
        WalkControl::Break
    }
}
