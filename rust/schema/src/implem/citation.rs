use stencila_codec_info::lost_options;

use crate::{Citation, CitationMode, prelude::*};

impl LatexCodec for Citation {
    fn to_latex(&self, context: &mut LatexEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id));

        // In render mode, use the pre-rendered content if available
        if context.render
            && let Some(content) = &self.options.content
        {
            content.to_latex(context);
            context.exit_node();
            return;
        }
        // Fall through to generate citation command as fallback

        // Determine the citation command based on mode
        let command = match self.citation_mode {
            Some(CitationMode::Narrative) => "citet",
            Some(CitationMode::NarrativeAuthor) => "citeauthor",
            Some(CitationMode::NarrativeYear) => "citeyear",
            _ => "citep", // Parenthetical is the default
        };

        context.char('\\').str(command);

        // Handle prefix and suffix (natbib style: \citep[prefix][suffix]{key})
        match (&self.options.citation_prefix, &self.options.citation_suffix) {
            (Some(prefix), Some(suffix)) => {
                context
                    .char('[')
                    .str(prefix)
                    .str("][")
                    .str(suffix)
                    .char(']');
            }
            (None, Some(suffix)) => {
                context.char('[').str(suffix).char(']');
            }
            (Some(prefix), None) => {
                context.char('[').str(prefix).str("][]");
            }
            (None, None) => {}
        }

        context
            .char('{')
            .property_str(NodeProperty::Target, &self.target)
            .char('}')
            .exit_node();
    }
}

impl JatsCodec for Citation {
    fn to_jats(&self, context: &mut JatsEncodeContext) {
        context.merge_losses(lost_options!(self, id));

        let parenthetical = matches!(self.citation_mode, Some(CitationMode::Parenthetical));
        if parenthetical {
            context.push_text("(");
        }

        // A citation can point at more than one reference, so each target is
        // resolved separately to the id that its reference is emitted with
        let target = self
            .target
            .split_whitespace()
            .map(|target| context.resolve_reference_id(target).to_string())
            .join(" ");
        context
            .enter_elem("xref")
            .push_attr("ref-type", "bibr")
            .push_attr("rid", target);
        if let Some(prefix) = &self.options.citation_prefix {
            context.push_text(prefix);
        }
        if let Some(inner) = &self.options.content {
            inner.to_jats(context);
        }
        if let Some(suffix) = &self.options.citation_suffix {
            context.push_text(suffix);
        }
        context.exit_elem();

        if parenthetical {
            context.push_text(")");
        }
    }
}

impl MarkdownCodec for Citation {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id));

        let brackets = matches!(self.citation_mode, None | Some(CitationMode::Parenthetical));

        if matches!(context.mode, MarkdownEncodeMode::Render) {
            if let Some(content) = &self.options.content {
                // Normally the citation will have content rendered in the citation
                // style so use that.
                content.to_markdown(context);
            } else {
                // Fallback to using the citation's target (within parentheses if appropriate)
                if brackets {
                    context.push_str("(").push_str(&self.target).push_str(")");
                } else {
                    context.push_str(&self.target);
                }
            }

            context.exit_node();
            return;
        }

        if brackets {
            context.push_str("[");
        }

        if let Some(prefix) = &self.options.citation_prefix {
            context.push_str(prefix);
        }

        context
            .push_str("@")
            .push_prop_str(NodeProperty::Target, &self.target);

        if let Some(suffix) = &self.options.citation_suffix {
            context.push_str(suffix);
        }

        if brackets {
            context.push_str("]");
        }

        context.exit_node();
    }
}

impl TextCodec for Citation {
    fn to_text(&self) -> String {
        // The rendered content, when the citation has been rendered in the document's
        // citation style. Otherwise the target, which is the citation's identity: a
        // citation with no rendered content is not nothing, and rendering it as an empty
        // string loses the only thing that distinguishes it from any other citation.
        match &self.options.content {
            Some(content) => content.to_text(),
            None => self.target.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use stencila_codec_text_trait::to_text;

    use crate::shortcuts::t;

    use super::*;

    /// A rendered citation reads as its rendering
    #[test]
    fn text_prefers_rendered_content() {
        let mut citation = Citation::new("smith2020".to_string());
        citation.options.content = Some(vec![t("Smith et al., 2020")]);

        assert_eq!(to_text(&citation), "Smith et al., 2020");
    }

    /// An unrendered citation reads as its target, rather than as nothing at all
    #[test]
    fn text_falls_back_to_the_target() {
        let citation = Citation::new("smith2020".to_string());

        assert_eq!(to_text(&citation), "smith2020");
    }
}
