use stencila_schema::{Heading, LabelType, NodeType};

use crate::{HeadingInfo, prelude::*};

impl Executable for Heading {
    #[tracing::instrument(skip_all)]
    async fn compile(&mut self, executor: &mut Executor) -> WalkControl {
        // If necessary, collapse previous headings into their parents
        HeadingInfo::collapse(self.level, &mut executor.headings);

        // If in the appendices, and the level is 1, then set the heading's label type
        // to AppendixLabel and reset table, figure, and equation counters
        if let Some(appendix_count) = &mut executor.appendix_count
            && self.level == 1
        {
            *appendix_count += 1;

            executor.figure_count = 0;
            executor.table_count = 0;
            executor.equation_count = 0;

            let label = executor.appendix_label();

            if !matches!(self.label_type, Some(LabelType::AppendixLabel))
                || self.label.as_ref() != Some(&label)
            {
                // Must be set locally for is and label registration (below)
                self.label_type = Some(LabelType::AppendixLabel);
                self.label = Some(label.clone());

                executor.patch(
                    &self.node_id(),
                    [
                        set(NodeProperty::LabelType, LabelType::AppendixLabel),
                        set(NodeProperty::Label, label),
                    ],
                );
            }
        }

        // If has id, label type and label may be a link target so register
        if let (Some(id), Some(label_type), Some(label)) = (&self.id, &self.label_type, &self.label)
        {
            executor
                .labels
                .insert(id.clone(), (*label_type, label.clone()));
        }

        // Record this heading if appropriate
        if !executor.walk_ancestors.iter().any(|node_type| {
            matches!(
                node_type,
                NodeType::Figure
                    | NodeType::Table
                    | NodeType::CodeChunk
                    | NodeType::Chat
                    | NodeType::PromptBlock
                    | NodeType::Excerpt
            )
        }) {
            let info = HeadingInfo {
                level: self.level,
                node_id: self.node_id(),
                content: self.content.clone(),
                children: Vec::new(),
            };
            executor.headings.push(info);
        }

        // Continue walk over content
        WalkControl::Continue
    }
}
