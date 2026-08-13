use stencila_codec_info::lost_options;

use crate::{MathBlock, MessageLevel, prelude::*};

use super::math::{encode_jats_math, encodes_as_jats_math};

impl MathBlock {
    pub fn has_warnings_errors_or_exceptions(&self) -> bool {
        self.options
            .compilation_messages
            .iter()
            .flatten()
            .any(|message| {
                matches!(
                    message.level,
                    MessageLevel::Warning | MessageLevel::Error | MessageLevel::Exception
                )
            })
    }
}

impl JatsCodec for MathBlock {
    fn to_jats(&self, context: &mut JatsEncodeContext) {
        // The math is emitted as JATS elements below; the source is only kept in
        // attributes of Stencila's own when JATS has no element for its language
        let recoverable = encodes_as_jats_math(self.math_language.as_deref(), &self.code);

        context.enter_elem("disp-formula");
        if !recoverable {
            context.push_attr("code", self.code.as_str());
        }
        if let Some(id) = &self.id {
            context.push_attr("id", id);
        }
        if let Some(lang) = &self.math_language
            && !recoverable
        {
            context.push_attr("language", lang);
        }

        if let Some(label) = &self.label {
            let label = if label.trim().starts_with('(') {
                label.to_string()
            } else {
                format!("({label})")
            };
            context.enter_elem("label").push_text(label).exit_elem();
        }

        encode_jats_math(
            &self.code,
            self.math_language.as_deref(),
            self.options.mathml.as_deref(),
            context,
        );

        for image in self.options.images.iter().flatten() {
            image.to_jats(context);
        }

        context.exit_elem().merge_losses(lost_options!(
            self.options,
            compilation_digest,
            compilation_messages
        ));
    }
}

impl DomCodec for MathBlock {
    fn to_dom(&self, context: &mut DomEncodeContext) {
        context.enter_node(self.node_type(), self.node_id());

        self.code.to_dom_attr("code", context);

        if let Some(math_language) = &self.math_language {
            context.push_attr("math-language", math_language);
        }

        if let Some(label) = &self.label {
            context.push_attr("label", label);
        }

        if let Some(label_automatically) = &self.label_automatically {
            context.push_attr("label-automatically", &label_automatically.to_string());
        }

        if let Some(id_automatically) = &self.id_automatically {
            context.push_attr("id-automatically", &id_automatically.to_string());
        }

        if let Some(id) = &self.id {
            context
                .enter_slot("div", "id")
                .push_attr("id", id)
                .exit_slot();
        }

        if let Some(messages) = &self.options.compilation_messages {
            context.push_slot_fn("div", "compilation-messages", |context| {
                messages.to_dom(context)
            });
        }

        if let Some(authors) = &self.authors {
            context.push_slot_fn("div", "authors", |context| authors.to_dom(context));
        }

        if let Some(provenance) = &self.provenance {
            context.push_slot_fn("div", "provenance", |context| provenance.to_dom(context));
        }

        if let Some(mathml) = &self.options.mathml {
            context.push_slot_fn("div", "mathml", |context| {
                context.push_html(mathml);
            });
        }

        if let Some(label) = &self.label {
            context.push_slot_fn("div", "label", |context| {
                context.push_html("(").push_text(label).push_html(")");
            });
        }

        if self.options.mathml.is_none()
            && let Some(images) = &self.options.images
        {
            context.push_slot_fn("div", "images", |context| images.to_dom(context));
        }

        context.exit_node();
    }
}

impl LatexCodec for MathBlock {
    fn to_latex(&self, context: &mut LatexEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id, math_language))
            .merge_losses(lost_options!(
                self.options,
                compilation_digest,
                compilation_messages,
                mathml
            ));

        if is_latex_display_math_environment(&self.code) {
            // Note: this intentionally does not escape code
            context
                .property_str(NodeProperty::Code, &self.code)
                .str(if self.code.ends_with('\n') { "" } else { "\n" });
        } else {
            context
                .str("\\[\n")
                // Note: this intentionally does not escape code
                .property_str(NodeProperty::Code, &self.code)
                .str(if self.code.ends_with('\n') {
                    "\\]\n"
                } else {
                    "\n\\]\n"
                });
        }

        context.exit_node().newline();
    }
}

fn is_latex_display_math_environment(code: &str) -> bool {
    const ENVS: [&str; 6] = [
        r"\begin{align}",
        r"\begin{align*}",
        r"\begin{gather}",
        r"\begin{gather*}",
        r"\begin{multline}",
        r"\begin{multline*}",
    ];

    let code = code.trim_start();

    ENVS.iter().any(|env| code.starts_with(env))
}

impl MarkdownCodec for MathBlock {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id, label, authors, provenance))
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
            // Add indentation for opening fence for SMD format
            if matches!(context.format, Format::Smd) {
                context.push_indent();
            }

            context
                .push_str("$$\n")
                .push_prop_fn(NodeProperty::Code, |context| self.code.to_markdown(context))
                .push_str(if self.code.ends_with('\n') { "" } else { "\n" });

            // Add indentation for closing fence for SMD format
            if matches!(context.format, Format::Smd) {
                context.push_indent();
            }

            context.push_str("$$");
        } else {
            // Add indentation for opening fence for SMD format
            if matches!(context.format, Format::Smd) {
                context.push_indent();
            }

            context
                .push_str("```")
                .push_prop_str(NodeProperty::MathLanguage, &lang)
                .newline()
                .push_prop_fn(NodeProperty::Code, |context| self.code.to_markdown(context))
                .push_str(if self.code.ends_with('\n') { "" } else { "\n" });

            // Add indentation for closing fence for SMD format
            if matches!(context.format, Format::Smd) {
                context.push_indent();
            }

            context.push_str("```");
        }

        context.newline().exit_node().newline();
    }
}
