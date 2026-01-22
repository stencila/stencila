use stencila_codec_info::lost_options;

use crate::{CitationGroup, CitationMode, prelude::*};

impl LatexCodec for CitationGroup {
    fn to_latex(&self, context: &mut LatexEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id));

        // In render mode, use the pre-rendered content if available
        if context.render
            && let Some(content) = &self.content
        {
            content.to_latex(context);
            context.exit_node();
            return;
        }
        // Fall through to generate citation commands as fallback

        // Check if all citations in the group have the same mode
        // If so, use a single command with multiple keys
        let first_mode = self.items.first().and_then(|c| c.citation_mode);
        let all_same_mode = self.items.iter().all(|c| {
            c.citation_mode == first_mode
                && c.options.citation_prefix.is_none()
                && c.options.citation_suffix.is_none()
        });

        if all_same_mode && !self.items.is_empty() {
            // All citations have same mode and no prefix/suffix, use single command
            let command = match first_mode {
                Some(CitationMode::Narrative) => "citet",
                Some(CitationMode::NarrativeAuthor) => "citeauthor",
                Some(CitationMode::NarrativeYear) => "citeyear",
                _ => "citep",
            };

            context.char('\\').str(command).char('{');

            for (index, citation) in self.items.iter().enumerate() {
                if index > 0 {
                    context.char(',');
                }
                context.str(&citation.target);
            }

            context.char('}');
        } else {
            // Mixed modes or prefix/suffix - encode each citation separately
            for citation in &self.items {
                citation.to_latex(context);
            }
        }

        context.exit_node();
    }
}

impl CitationGroup {
    pub fn to_jats_special(&self) -> (String, Losses) {
        (
            [
                "(",
                &self
                    .items
                    .iter()
                    .map(|item| item.to_jats_special().0)
                    .join("; "),
                ")",
            ]
            .concat(),
            Losses::none(),
        )
    }
}

impl MarkdownCodec for CitationGroup {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id));

        if context.render {
            if let Some(content) = &self.content {
                // Normally the citation group will have content rendered in the citation
                // style so use that.
                content.to_markdown(context);
            } else {
                // Fallback to using the citations' target
                context.push_str("(");
                for (index, citation) in self.items.iter().enumerate() {
                    if index > 0 {
                        context.push_str("; ");
                    }
                    context.push_str(&citation.target);
                }
                context.push_str(")");
            }
            context.exit_node();
            return;
        }

        context.push_str("[");

        for (index, item) in self.items.iter().enumerate() {
            if index > 0 {
                context.push_str("; ");
            }
            item.to_markdown(context);
        }

        context.push_str("]").exit_node();
    }
}
