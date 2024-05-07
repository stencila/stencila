use codec_info::lost_options;

use crate::{prelude::*, CodeInline};

impl DomCodec for CodeInline {
    fn to_dom(&self, context: &mut DomEncodeContext) {
        context.enter_node(self.node_type(), self.node_id());

        // WebComponents use this `code` attribute to render code with syntax highlighting
        self.code.to_dom_attr("code", context);

        if let Some(programming_language) = &self.programming_language {
            context.push_attr("programming-language", programming_language);
        }

        if let Some(authors) = &self.authors {
            context.push_slot_fn("span", "authors", |context| authors.to_dom(context));
        }

        if let Some(provenance) = &self.provenance {
            context.push_slot_fn("span", "provenance", |context| provenance.to_dom(context));
        }

        // Put code in a `<code>` as well so that it is visible in static view.
        context.enter_elem("code").push_text(&self.code).exit_elem();

        context.exit_node();
    }
}

impl MarkdownCodec for CodeInline {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id));

        context
            .push_str("`")
            .push_prop_fn(NodeProperty::Code, |context| self.code.to_markdown(context))
            .push_str("`");

        if let Some(lang) = &self.programming_language {
            context
                .push_str("{")
                .push_prop_str(NodeProperty::ProgrammingLanguage, &lang.replace('}', r"\}"))
                .push_str("}");
        }

        context.exit_node();
    }
}
