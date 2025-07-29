use codec_info::lost_options;

use crate::{Heading, LabelType, prelude::*};

impl Heading {
    pub fn to_jats_special(&self) -> (String, Losses) {
        use codec_jats_trait::encode::elem;

        let (content, mut losses) = self.content.to_jats();

        // The `level` attribute is not part of the JATS standard but allows
        // lossless roundtrip conversion of Stencila documents to/from JATS.
        let attrs = [("level", &self.level)];

        if self.id.is_some() {
            losses.add("Heading.id")
        }

        (elem("title", attrs, content), losses)
    }

    pub fn to_html_special(&self, context: &mut HtmlEncodeContext) -> String {
        use codec_html_trait::encode::{attr, elem};
        elem(
            &["h", &self.level.clamp(1, 6).to_string()].concat(),
            &[attr("id", &self.id.to_html_attr(context))],
            &[self.content.to_html(context)],
        )
    }
}

impl DomCodec for Heading {
    fn to_dom(&self, context: &mut DomEncodeContext) {
        context
            .enter_heading(self.level, self.node_id())
            .push_id(&self.id)
            .push_attr("level", &self.level.to_string())
            .push_slot_fn(
                &["h", &self.level.clamp(1, 6).to_string()].concat(),
                "content",
                |context| {
                    if matches!(self.label_type, Some(LabelType::AppendixLabel))
                        || self.label.is_some()
                    {
                        context.enter_elem_attrs("span", [("class", "heading-label")]);

                        if self.label.is_some() {
                            context.push_text("Appendix ");
                        }

                        if let Some(label) = &self.label {
                            context.push_text(label).push_text(" ");
                        }

                        context.exit_elem();
                    }
                    self.content.to_dom(context)
                },
            );

        if let Some(authors) = &self.authors {
            context.push_slot_fn("div", "authors", |context| authors.to_dom(context));
        }

        if let Some(provenance) = &self.provenance {
            context.push_slot_fn("div", "provenance", |context| provenance.to_dom(context));
        }

        context.exit_node();
    }
}

impl LatexCodec for Heading {
    fn to_latex(&self, context: &mut LatexEncodeContext) {
        let command = match self.level {
            1 => "section",
            2 => "subsection",
            3 => "subsubsection",
            4 => "paragraph",
            _ => "subparagraph",
        };

        context
            .ensure_blankline()
            .enter_node(self.node_type(), self.node_id())
            .command_begin(command);

        if self.level == 1
            && matches!(self.label_type, Some(LabelType::AppendixLabel))
            && context.has_format_via_pandoc()
        {
            if self.label_type.is_some() {
                context.str("Appendix ");
            }
            if let Some(label) = &self.label {
                context.str(label).char(' ');
            }
        }

        context
            .property_fn(NodeProperty::Content, |context| {
                self.content.to_latex(context)
            })
            .command_end()
            .newline();

        // Add id (if any) as a label to that cross links work
        if let Some(id) = &self.id {
            context.str(r"\label{").str(id).str("}\n");
        }

        context.exit_node().newline();
    }
}

impl MarkdownCodec for Heading {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id, authors, provenance))
            .push_prop_str(
                NodeProperty::Level,
                &"#".repeat(self.level.clamp(1, 6) as usize),
            )
            .push_str(" ");

        if context.render {
            if matches!(self.label_type, Some(LabelType::AppendixLabel)) {
                context
                    .push_prop_str(NodeProperty::LabelType, "Appendix")
                    .push_str(" ");
            }
            if let Some(label) = &self.label {
                context
                    .push_prop_str(NodeProperty::Label, label)
                    .push_str(" ");
            }
        }

        context
            .push_prop_fn(NodeProperty::Content, |context| {
                self.content.to_markdown(context)
            })
            .newline()
            .exit_node()
            .newline();
    }
}
