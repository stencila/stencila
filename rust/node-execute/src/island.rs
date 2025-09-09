use stencila_schema::{Island, LabelType, NodeProperty};

use crate::prelude::*;

impl Executable for Island {
    #[tracing::instrument(skip_all)]
    async fn compile(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::trace!("Compiling Island {node_id}");

        if let Some(label_type) = &self.label_type {
            // Update executor figure/table count and label
            let label = match label_type {
                LabelType::FigureLabel => executor.figure_label(),
                LabelType::TableLabel => executor.table_label(),
                LabelType::AppendixLabel => executor.appendix_label(),
                LabelType::SupplementLabel => executor.supplement_label(),
            };

            if self.label_automatically.unwrap_or(true) && Some(&label) != self.label.as_ref() {
                self.label = Some(label.clone());
                executor.patch(&node_id, [set(NodeProperty::Label, label)]);
            }
        }

        // If has id, label type and label may be a link target so register
        if let (Some(id), Some(label_type), Some(label)) = (&self.id, &self.label_type, &self.label)
        {
            executor
                .labels
                .insert(id.clone(), (*label_type, label.clone()));
        }

        WalkControl::Continue
    }
}
