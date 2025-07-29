use schema::{LabelType, Link, shortcuts::t};

use crate::prelude::*;

impl Executable for Link {
    async fn link(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::trace!("Linking Link {node_id}");

        // Update the content of the link if it has an internal target
        if !(self.target.starts_with("https://") || self.target.starts_with("http://")) {
            if let Some((label_type, label)) = executor.labels.get(&self.target) {
                let label_type = match label_type {
                    LabelType::TableLabel => "Table",
                    LabelType::FigureLabel => "Figure",
                    LabelType::AppendixLabel => "Appendix",
                };

                let content = if self.label_only.unwrap_or_default() {
                    label.clone()
                } else {
                    [label_type, " ", label].concat()
                };
                let content = vec![t(content)];

                self.content = content.clone();
                executor.patch(
                    &node_id,
                    [
                        clear(NodeProperty::Content),
                        append(NodeProperty::Content, content),
                    ],
                );
            }
        }

        WalkControl::Continue
    }
}
