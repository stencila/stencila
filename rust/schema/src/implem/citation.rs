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

        let target = context.resolve_reference_id(&self.target).to_string();
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
