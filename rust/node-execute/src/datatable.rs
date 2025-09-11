use stencila_schema::{Datatable, LabelType, NodeProperty};

use crate::prelude::*;

impl Executable for Datatable {
    #[tracing::instrument(skip_all)]
    async fn compile(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::trace!("Compiling Datatable {node_id}");

        // Update automatic label if necessary
        if self.label_automatically.unwrap_or(true) {
            let label = executor.table_label();
            if Some(&label) != self.label.as_ref() {
                self.label = Some(label.clone());
                executor.patch(&node_id, [set(NodeProperty::Label, label)]);
            }
        }

        // If have id and label then register as a link target
        if let (Some(id), Some(label)) = (&self.id, &self.label) {
            executor
                .labels
                .insert(id.clone(), (LabelType::TableLabel, label.clone()));
        }

        WalkControl::Continue
    }
}
