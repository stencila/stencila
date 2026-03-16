use stencila_schema::{Island, LabelType, NodeProperty};

use crate::prelude::*;

impl Executable for Island {
    #[tracing::instrument(skip_all)]
    async fn compile(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::trace!("Compiling Island {node_id}");

        if let Some(label_type) = &self.label_type {
            // For continuation islands, reuse the current label without
            // incrementing. Appendix labels are always read-only (no counter
            // to increment), so they take the same path regardless.
            let is_continuation = self.is_continuation.unwrap_or(false);
            let label = if is_continuation
                && !matches!(label_type, LabelType::AppendixLabel)
                && executor.has_prior_label(label_type)
            {
                match label_type {
                    LabelType::FigureLabel => executor.figure_label_continued(),
                    LabelType::TableLabel => executor.table_label_continued(),
                    LabelType::SupplementLabel => executor.supplement_label_continued(),
                    LabelType::AppendixLabel => unreachable!(),
                }
            } else {
                if is_continuation && !matches!(label_type, LabelType::AppendixLabel) {
                    tracing::warn!(
                        "Island {node_id} has isContinuation but no prior {label_type:?}; \
                         falling back to normal numbering"
                    );
                }
                match label_type {
                    LabelType::FigureLabel => executor.figure_label(),
                    LabelType::TableLabel => executor.table_label(),
                    LabelType::AppendixLabel => executor.appendix_label(),
                    LabelType::SupplementLabel => executor.supplement_label(),
                }
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
