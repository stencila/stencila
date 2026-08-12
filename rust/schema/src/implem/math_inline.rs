use stencila_codec_info::lost_options;

use crate::{MathInline, prelude::*};

impl JatsCodec for MathInline {
    fn to_jats(&self, context: &mut JatsEncodeContext) {
        context
            .enter_elem("inline-formula")
            .push_attr("code", self.code.as_str());
        if let Some(lang) = &self.math_language {
            context.push_attr("language", lang);
        }

        if let Some(mathml) = &self.options.mathml {
            context.enter_elem("mml:math").push_xml(mathml).exit_elem();
        }

        context
            .exit_elem()
            .merge_losses(lost_options!(self, id))
            .merge_losses(lost_options!(
                self.options,
                compilation_digest,
                compilation_messages
            ));
    }
}

impl DomCodec for MathInline {
    fn to_dom(&self, context: &mut DomEncodeContext) {
        context.enter_node(self.node_type(), self.node_id());

        self.code.to_dom_attr("code", context);

        if let Some(math_language) = &self.math_language {
            context.push_attr("math-language", math_language);
        }

        if let Some(messages) = &self.options.compilation_messages {
            context.push_slot_fn("span", "compilation-messages", |context| {
                messages.to_dom(context)
            });
        }

        if let Some(authors) = &self.authors {
            context.push_slot_fn("span", "authors", |context| authors.to_dom(context));
        }

        if let Some(provenance) = &self.provenance {
            context.push_slot_fn("span", "provenance", |context| provenance.to_dom(context));
        }

        if let Some(mathml) = &self.options.mathml {
            context.push_slot_fn("span", "mathml", |context| {
                context.push_html(mathml);
            });
        }

        if self.options.mathml.is_none()
            && let Some(images) = &self.options.images
        {
            context.push_slot_fn("span", "images", |context| images.to_dom(context));
        }

        context.exit_node();
    }
}

impl LatexCodec for MathInline {
    fn to_latex(&self, context: &mut LatexEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id, math_language))
            .merge_losses(lost_options!(
                self.options,
                compilation_digest,
                compilation_messages,
                mathml
            ))
            .str("\\(")
            // Note: this intentionally does not escape code
            .property_str(NodeProperty::Code, &self.code)
            .str("\\)")
            .exit_node();
    }
}

impl MarkdownCodec for MathInline {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id))
            .merge_losses(lost_options!(
                self.options,
                compilation_digest,
                compilation_messages,
                mathml
            ));

        let lang = self
            .math_language
            .as_deref()
            .unwrap_or("tex")
            .to_lowercase();

        if lang == "tex" || lang == "latex" || lang == "math" {
            context
                .push_str("$")
                .push_prop_str(NodeProperty::Code, &self.code.replace('$', r"\$"))
                .push_str("$");
        } else {
            context
                .push_str("`")
                .push_prop_str(NodeProperty::Code, &self.code.replace('`', r"\`"))
                .push_str("`{")
                .push_prop_str(NodeProperty::MathLanguage, &lang)
                .push_str("}");
        }

        context.exit_node();
    }
}
