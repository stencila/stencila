use codec_info::lost_options;

use crate::{Citation, CitationMode, prelude::*};

impl Citation {
    pub fn to_jats_special(&self) -> (String, Losses) {
        use codec_jats_trait::encode::elem;

        let mut losses = lost_options!(self, id);

        let attrs = vec![("ref-type", "bibr"), ("rid", &self.target)];

        let mut content = String::new();
        if let Some(prefix) = &self.options.citation_prefix {
            content.push_str(prefix);
        }
        if let Some(inner) = &self.options.content {
            let (inner, inner_losses) = inner.to_jats();
            content.push_str(&inner);
            losses.merge(inner_losses);
        }
        if let Some(suffix) = &self.options.citation_suffix {
            content.push_str(suffix);
        }

        let xref = elem("xref", attrs, content);

        let jats = if matches!(self.citation_mode, Some(CitationMode::Parenthetical)) {
            ["(", &xref, ")"].concat()
        } else {
            xref
        };

        (jats, losses)
    }
}

impl MarkdownCodec for Citation {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        let brackets = matches!(self.citation_mode, Some(CitationMode::Parenthetical));

        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id));

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
