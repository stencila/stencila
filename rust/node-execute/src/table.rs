use schema::{NodeProperty, Table};

use crate::prelude::*;

impl Executable for Table {
    #[tracing::instrument(skip_all)]
    async fn compile(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::trace!("Compiling Table {node_id}");

        executor.table_count += 1;

        if self.label_automatically.unwrap_or(true) {
            let label = executor.table_count.to_string();
            if Some(&label) != self.label.as_ref() {
                executor.patch(&node_id, [set(NodeProperty::Label, label)]);
            }
        }

        WalkControl::Continue
    }

    #[tracing::instrument(skip_all)]
    async fn prepare(&mut self, executor: &mut Executor) -> WalkControl {
        tracing::trace!("Preparing Table {}", self.node_id());

        // Begin adding the table to the document context
        executor.document_context.begin_table((&*self).into());

        // Walk over caption, rows and notes
        if let Err(error) = async {
            self.caption.walk_async(executor).await?;
            self.rows.walk_async(executor).await?;
            self.notes.walk_async(executor).await
        }
        .await
        {
            tracing::error!("While preparing table: {error}")
        }

        // End adding the table to the context
        executor.document_context.end_table();

        // Break walk because properties prepared above
        WalkControl::Break
    }

    #[tracing::instrument(skip_all)]
    async fn execute(&mut self, executor: &mut Executor) -> WalkControl {
        tracing::trace!("Executing Table {}", self.node_id());

        // Enter the table context
        executor.document_context.enter_table();

        // Walk over caption, rows and notes
        if let Err(error) = async {
            self.caption.walk_async(executor).await?;
            self.rows.walk_async(executor).await?;
            self.notes.walk_async(executor).await
        }
        .await
        {
            tracing::error!("While executing table: {error}")
        }

        // Exit the table context
        executor.document_context.exit_table();

        // Break walk because properties executed above
        WalkControl::Break
    }
}
