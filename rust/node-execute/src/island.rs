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

        // Register all label IDs as link targets (including subfigure/subtable labels)
        if let (Some(label_type), Some(label)) = (&self.label_type, &self.label) {
            // If other_ids is set, register all of them
            if let Some(other_ids) = &self.other_ids {
                for label_id in other_ids {
                    executor
                        .labels
                        .insert(label_id.clone(), (*label_type, label.clone()));
                }
            } else if let Some(id) = &self.id {
                // Fallback to just the primary id if other_ids is not set
                executor
                    .labels
                    .insert(id.clone(), (*label_type, label.clone()));
            }
        }

        WalkControl::Continue
    }
}
