use schema::{Figure, NodeProperty};

use crate::prelude::*;

impl Executable for Figure {
    #[tracing::instrument(skip_all)]
    async fn compile(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::trace!("Compiling Figure {node_id}");

        let label = executor.figure_label();

        if self.label_automatically.unwrap_or(true) && Some(&label) != self.label.as_ref() {
            executor.patch(&node_id, [set(NodeProperty::Label, label)]);
        }

        WalkControl::Continue
    }

    #[tracing::instrument(skip_all)]
    async fn prepare(&mut self, executor: &mut Executor) -> WalkControl {
        tracing::trace!("Preparing Figure {}", self.node_id());

        // Begin adding the figure to the document context
        executor.document_context.begin_figure((&*self).into());

        // Walk over caption and content
        if let Err(error) = async {
            self.caption.walk_async(executor).await?;
            self.content.walk_async(executor).await
        }
        .await
        {
            tracing::error!("While preparing figure: {error}")
        }

        // End adding the figure to the context
        executor.document_context.end_figure();

        // Break walk because properties prepared above
        WalkControl::Break
    }

    #[tracing::instrument(skip_all)]
    async fn execute(&mut self, executor: &mut Executor) -> WalkControl {
        tracing::trace!("Executing Figure {}", self.node_id());

        // Enter the figure context
        executor.document_context.enter_figure();

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
        executor.document_context.exit_figure();

        // Break walk because content executed above
        WalkControl::Break
    }
}
