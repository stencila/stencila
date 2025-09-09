use stencila_schema::{LabelType, NodeProperty, Supplement};

use crate::prelude::*;

impl Executable for Supplement {
    #[tracing::instrument(skip_all)]
    async fn compile(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::trace!("Compiling Supplement {node_id}");

        // Update automatic label if necessary
        if self.label_automatically.unwrap_or(true) {
            let label = executor.supplement_label();
            if Some(&label) != self.label.as_ref() {
                self.label = Some(label.clone());
                executor.patch(&node_id, [set(NodeProperty::Label, label)]);
            }
        }

        // If have id and label then register as a link target
        if let (Some(id), Some(label)) = (&self.id, &self.label) {
            executor
                .labels
                .insert(id.clone(), (LabelType::SupplementLabel, label.clone()));
        }

        // Compile the work, if a new executor (so that headings, references
        // etc are not collected and numbering of figures etc is reset)
        if let Some(work) = &mut self.options.work {
            let mut fork = executor.fork_for_supplement();
            if let Err(error) = work.walk_async(&mut fork).await {
                tracing::error!("While compiling supplement work: {error}")
            }
        }

        WalkControl::Continue
    }
}
