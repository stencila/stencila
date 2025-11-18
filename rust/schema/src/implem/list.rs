use stencila_codec_html_trait::encode::elem;
use stencila_codec_info::lost_options;

use crate::{List, ListOrder, prelude::*};

impl List {
    pub fn to_html_special(&self, context: &mut HtmlEncodeContext) -> String {
        let tag = match &self.order {
            ListOrder::Ascending => "ol",
            _ => "ul",
        };

        let items = self.items.to_html(context);

        elem(tag, &[], &[items])
    }
}

impl DomCodec for List {
    fn to_dom(&self, context: &mut DomEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .push_id(&self.id)
            .push_attr("order", &self.order.to_string())
            .push_slot_fn(
                if matches!(self.order, ListOrder::Ascending) {
                    "ol"
                } else {
                    "ul"
                },
                "items",
                |context| self.items.to_dom(context),
            );

        if let Some(authors) = &self.authors {
            context.push_slot_fn("div", "authors", |context| authors.to_dom(context));
        }

        context.exit_node();
    }
}

impl LatexCodec for List {
    fn to_latex(&self, context: &mut LatexEncodeContext) {
        context.enter_node(self.node_type(), self.node_id());

        let unordered = matches!(self.order, ListOrder::Unordered);
        let environ = if unordered { "itemize" } else { "enumerate" };

        context.environ_begin(environ).newline();

        if !unordered {
            context.str(r"\def\labelenumi{\arabic{enumi}.}").newline();
        }

        let tight = self.items.iter().all(|item| item.content.len() == 1);

        for item in self.items.iter() {
            item.to_latex(context);

            if tight {
                context.trim_end().newline();
            }
        }

        context
            .trim_end()
            .newline()
            .environ_end(environ)
            .exit_node()
            .newline();
    }
}

impl MarkdownCodec for List {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id, authors, provenance));

        let ordered = matches!(self.order, ListOrder::Ascending);

        let tight = self.items.iter().all(|item| item.content.len() == 1);

        for (index, item) in self.items.iter().enumerate() {
            // Add indentation for SMD format
            if matches!(context.format, Format::Smd) {
                context.push_indent();
            }

            if ordered {
                context.push_str(&(index + 1).to_string()).push_str(". ")
            } else {
                context.push_str("- ")
            };

            context.push_line_prefix("  ");
            item.to_markdown(context);
            context.pop_line_prefix();

            if tight {
                context.trim_end().newline();
            }
        }

        context.trim_end().newline().exit_node().newline();
    }
}
