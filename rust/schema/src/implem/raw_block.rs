use codec_info::lost_options;

use crate::{prelude::*, RawBlock};

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

        // Add a div for the content if HTML or SVG
        let format = Format::from_name(&self.format);
        if matches!(format, Format::Html | Format::Svg) {
            context.push_slot_fn("div", "content", |context| match format {
                Format::Html | Format::Svg => {
                    context.push_html(&self.content.string);
                }
                _ => {}
            });
        }

        context.exit_node();
    }
}

impl MarkdownCodec for RawBlock {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        if context.render || matches!(context.format, Format::Llmd) {
            // Encode content if format of RawBlock is any Markdown flavor
            if Format::from_name(&self.format).is_markdown_flavor() {
                context.push_str(&self.content);

                // Add as many newlines to separate from following blocks
                if !self.content.ends_with('\n') {
                    context.newline();
                }
                context.newline();

                return;
            }
        }

        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id));

        context
            .push_str("````")
            .push_prop_str(NodeProperty::Format, &self.format)
            .push_str(" raw\n")
            .push_prop_fn(NodeProperty::Code, |context| {
                self.content.to_markdown(context)
            });

        if !self.content.ends_with('\n') {
            context.newline();
        }

        context.push_str("````\n").exit_node().newline();
    }
}
