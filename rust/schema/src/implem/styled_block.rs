use stencila_codec_info::lost_options;

use crate::{StyledBlock, prelude::*};

impl DomCodec for StyledBlock {
    fn to_dom(&self, context: &mut DomEncodeContext) {
        context.enter_node(self.node_type(), self.node_id());

        self.code.to_dom_attr("code", context);

        if let Some(style_language) = &self.style_language {
            context.push_attr("style-language", style_language);
        }

        if let Some(css) = &self.options.css {
            context.push_css(css);
        };

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

        context.push_slot_fn("div", "content", |context| {
            if let Some(class) = &self.options.class_list {
                context.push_attr("class", class);
            };
            self.content.to_dom(context)
        });

        context.exit_node();
    }
}

impl LatexCodec for StyledBlock {
    fn to_latex(&self, context: &mut LatexEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id))
            .add_loss("StyledBlock.code")
            .property_fn(NodeProperty::Content, |context| {
                self.content.to_latex(context)
            })
            .exit_node();
    }
}

impl MarkdownCodec for StyledBlock {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id, style_language))
            .merge_losses(lost_options!(
                self.options,
                compilation_digest,
                compilation_messages,
                css,
                class_list
            ));

        // If rendering, or format is anything other than Stencila or Quarto
        // Markdown, then encode `content` only (if any)
        if context.render || !matches!(context.format, Format::Smd | Format::Qmd) {
            context.push_prop_fn(NodeProperty::Content, |context| {
                self.content.to_markdown(context)
            });

            context.exit_node();
            return;
        }

        if matches!(context.format, Format::Qmd) {
            context
                .push_str("::: {")
                .push_prop_fn(NodeProperty::Code, |context| self.code.to_markdown(context))
                .push_str("}\n\n")
                .push_prop_fn(NodeProperty::Content, |context| {
                    self.content.to_markdown(context)
                })
                .push_str(":::");
        } else {
            context
                .push_colons()
                .push_str(" style ")
                .push_prop_fn(NodeProperty::Code, |context| self.code.to_markdown(context))
                .push_str("\n\n")
                .increase_depth()
                .push_prop_fn(NodeProperty::Content, |context| {
                    self.content.to_markdown(context)
                })
                .decrease_depth()
                .push_colons();
        }

        context.newline().exit_node().newline();
    }
}
