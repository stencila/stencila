use stencila_codec_info::lost_options;

use crate::{RawBlock, prelude::*};

impl DomCodec for RawBlock {
    fn to_dom(&self, context: &mut DomEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .push_attr("format", &self.format);

        // Use `Cord::to_dom_attr` here to get both "content" and "content-authorship" attributes
        self.content.to_dom_attr("content", context);

        if let Some(messages) = &self.compilation_messages {
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

        // Push any CSS to the context so that it can be put in the right place
        // in the document (usually only applies to HTML and CSS)
        if let Some(css) = &self.css {
            context.push_css(css);
        }

        let format = Format::from_name(&self.format);

        // Add a div for the content if HTML or SVG
        if matches!(format, Format::Html | Format::Svg) {
            context.push_slot_fn("div", "content", |context| {
                context.push_html(&self.content);
            });
        }

        context.exit_node();
    }
}

impl LatexCodec for RawBlock {
    fn to_latex(&self, context: &mut LatexEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id))
            .add_loss("RawBlock.format")
            // Note: this intentionally does not escape content
            .property_str(NodeProperty::Content, &self.content)
            .exit_node();
    }
}

impl MarkdownCodec for RawBlock {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id));

        // If rendering, or if format is LLM Markdown, encode `content` if
        // `format` is any Markdown flavor, HTML or LaTeX
        if context.render || matches!(context.format, Format::Llmd) {
            let format = Format::from_name(&self.format);
            if format.is_markdown_flavor() || matches!(format, Format::Html | Format::Latex) {
                context.push_str(&self.content);

                // Add as many newlines to separate from following blocks
                if !self.content.ends_with('\n') {
                    context.newline();
                }
                context.newline();

                return;
            }
        }

        match context.format {
            Format::Myst => {
                context
                    .push_str("````{raw} ")
                    .push_prop_str(NodeProperty::Format, &self.format)
                    .push_str("\n");
            }
            Format::Qmd => {
                context
                    .push_str("````{=")
                    .push_prop_str(NodeProperty::Format, &self.format)
                    .push_str("}\n");
            }
            Format::Smd => {
                context
                    .push_str("````")
                    .push_prop_str(NodeProperty::Format, &self.format)
                    .push_str(" raw\n");
            }
            _ => {}
        }

        context.push_prop_fn(NodeProperty::Code, |context| {
            self.content.to_markdown(context)
        });

        if matches!(context.format, Format::Myst | Format::Qmd | Format::Smd) {
            if !self.content.ends_with('\n') {
                context.newline();
            }
            context.push_str("````\n");
        }

        context.exit_node().newline();
    }
}
