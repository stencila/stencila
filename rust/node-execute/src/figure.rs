use schema::{Figure, NodeProperty};

use crate::prelude::*;

impl Executable for Figure {
    #[tracing::instrument(skip_all)]
    async fn compile(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::trace!("Compiling Figure {node_id}");

        executor.figure_count += 1;

        if self.label_automatically.unwrap_or(true) {
            let label = executor.figure_count.to_string();
            if Some(&label) != self.label.as_ref() {
                executor.patch(&node_id, [set(NodeProperty::Label, label)]);
            }
        }

        WalkControl::Continue
    }

    #[tracing::instrument(skip_all)]
    async fn prepare(&mut self, executor: &mut Executor) -> WalkControl {
        tracing::trace!("Preparing Figure {}", self.node_id());

        // Add figures to document context
        executor.document_context.figures.push((&*self).into());

        // Continue walk over caption and rows
        WalkControl::Continue
    }

    #[tracing::instrument(skip_all)]
    async fn execute(&mut self, executor: &mut Executor) -> WalkControl {
        tracing::trace!("Executing Figure {}", self.node_id());

        // Enter the figure context
        executor.document_context.figures.enter();

        // Walk over caption and content
        if let Err(error) = async {
            self.caption.walk_async(executor).await?;
            self.content.walk_async(executor).await
        }
        .await
        {
            tracing::error!("While executing figure: {error}")
        }

        // Exit the figure context
        executor.document_context.figures.exit();

        // Break walk because content executed above
        WalkControl::Break
    }
}
