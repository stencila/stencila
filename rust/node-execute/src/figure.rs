use stencila_schema::{Figure, LabelType, NodeProperty};

use crate::prelude::*;

impl Executable for Figure {
    #[tracing::instrument(skip_all)]
    async fn compile(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::trace!("Compiling Figure {node_id}");

        // Update automatic label if necessary.
        // Figures nested inside another figure get subfigure labels (e.g. "1A")
        // rather than incrementing the top-level figure count.
        if self.label_automatically.unwrap_or(true) {
            let label = if executor.has_figure_ancestor() {
                executor.subfigure_label()
            } else {
                executor.figure_label()
            };
            if Some(&label) != self.label.as_ref() {
                self.label = Some(label.clone());
                executor.patch(&node_id, [set(NodeProperty::Label, label)]);
            }
        }

        // Auto-generate an id from the label if the figure doesn't already
        // have a user-supplied id
        if let Some(label) = &self.label
            && let Some(id) = Executor::auto_id(&LabelType::FigureLabel, label, &self.id)
        {
            self.id = Some(id.clone());
            executor.patch(&node_id, [set(NodeProperty::Id, id)]);
        }

        // If have id and label then register as a link target
        if let (Some(id), Some(label)) = (&self.id, &self.label) {
            executor
                .labels
                .insert(id.clone(), (LabelType::FigureLabel, label.clone()));
        }

        WalkControl::Continue
    }
}
