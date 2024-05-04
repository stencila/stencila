use codec_html_trait::encode::elem;
use codec_info::lost_options;

use crate::{prelude::*, List, ListOrder};

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

impl MarkdownCodec for List {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id, authors, provenance));

        let ordered = matches!(self.order, ListOrder::Ascending);

        let tight = self.items.iter().all(|item| item.content.len() == 1);

        for (index, item) in self.items.iter().enumerate() {
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
