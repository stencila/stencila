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

impl JatsCodec for CitationGroup {
    fn to_jats(&self, context: &mut JatsEncodeContext) {
        context.push_text("(");
        for (index, item) in self.items.iter().enumerate() {
            if index > 0 {
                context.push_text("; ");
            }
            item.to_jats(context);
        }
        context.push_text(")");
    }
}

impl MarkdownCodec for CitationGroup {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id));

        if matches!(context.mode, MarkdownEncodeMode::Render) {
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

impl TextCodec for CitationGroup {
    fn to_text(&self) -> String {
        // The rendered content, when the group has been rendered in the document's
        // citation style, as for a single citation
        if let Some(content) = &self.content {
            return content.to_text();
        }

        // Otherwise the same shape the JATS and Markdown encodings give a group: its
        // items, separated by semicolons, in parentheses
        let items = self
            .items
            .iter()
            .map(|item| item.to_text())
            .collect::<Vec<_>>()
            .join("; ");

        ["(", &items, ")"].concat()
    }
}

#[cfg(test)]
mod tests {
    use stencila_codec_text_trait::to_text;

    use crate::{Citation, shortcuts::t};

    use super::*;

    /// An unrendered group reads as its items, separated by semicolons, in parentheses
    #[test]
    fn text_joins_the_items() {
        let group = CitationGroup::new(vec![
            Citation::new("smith2020".to_string()),
            Citation::new("jones2021".to_string()),
        ]);

        assert_eq!(to_text(&group), "(smith2020; jones2021)");
    }

    /// Each item resolves itself, so a rendered item contributes its rendering
    #[test]
    fn text_uses_each_item_s_own_rendering() {
        let mut rendered = Citation::new("smith2020".to_string());
        rendered.options.content = Some(vec![t("Smith 2020")]);

        let group = CitationGroup::new(vec![rendered, Citation::new("jones2021".to_string())]);

        assert_eq!(to_text(&group), "(Smith 2020; jones2021)");
    }

    /// A group rendered as a whole reads as that rendering
    #[test]
    fn text_prefers_rendered_content() {
        let mut group = CitationGroup::new(vec![Citation::new("smith2020".to_string())]);
        group.content = Some(vec![t("(Smith 2020)")]);

        assert_eq!(to_text(&group), "(Smith 2020)");
    }
}
