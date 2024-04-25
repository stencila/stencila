use codec_info::lost_options;

use crate::{prelude::*, MathBlock};

impl MathBlock {
    pub fn to_jats_special(&self) -> (String, Losses) {
        use codec_jats_trait::encode::{elem, elem_no_attrs};

        let label = self
            .label
            .as_ref()
            .map(|label| elem_no_attrs("label", label))
            .unwrap_or_default();

        let mathml = self
            .options
            .mathml
            .as_ref()
            .map(|mathml| elem_no_attrs("mml:math", mathml))
            .unwrap_or_default();

        let mut attrs = vec![("code", self.code.as_str())];
        if let Some(lang) = &self.math_language {
            attrs.push(("language", lang.as_str()));
        }

        let jats = elem("disp-formula", attrs, [label, mathml].concat());

        let mut losses = lost_options!(self, id);
        losses.merge(lost_options!(
            self.options,
            compilation_digest,
            compilation_messages
        ));

        (jats, losses)
    }
}

impl DomCodec for MathBlock {
    fn to_dom(&self, context: &mut DomEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .push_attr("code", &self.code);

        if let Some(math_language) = &self.math_language {
            context.push_attr("math-language", math_language);
        }

        if let Some(messages) = &self.options.compilation_messages {
            context.push_slot_fn("div", "compilation-messages", |context| {
                messages.to_dom(context)
            });
        }

        if let Some(authors) = &self.options.authors {
            context.push_slot_fn("div", "authors", |context| authors.to_dom(context));
        }

        if let Some(mathml) = &self.options.mathml {
            context.push_slot_fn("div", "mathml", |context| {
                context.push_html(mathml);
            });
        }

        context.exit_node();
    }
}

impl MarkdownCodec for MathBlock {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id, label))
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
                .push_str("$$\n")
                .push_prop_str(NodeProperty::Code, &self.code)
                .push_str(if self.code.ends_with('\n') { "" } else { "\n" })
                .push_str("$$");
        } else {
            context
                .push_str("```")
                .push_prop_str(NodeProperty::MathLanguage, &lang)
                .newline()
                .push_prop_str(NodeProperty::Code, &self.code)
                .push_str(if self.code.ends_with('\n') { "" } else { "\n" })
                .push_str("```");
        }

        context.newline().exit_node().newline();
    }
}
