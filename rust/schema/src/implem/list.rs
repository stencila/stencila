use codec_html_trait::encode::elem;
use codec_losses::lost_options;

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

impl MarkdownCodec for List {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id));

        context.push_str("\n").exit_node().push_str("\n");

        /*
         let ordered = matches!(self.order, ListOrder::Ascending);

         let items: Vec<String> = self
             .items
             .iter()
             .enumerate()
             .map(|(index, item)| {
                 let bullet = if ordered {
                     (index + 1).to_string() + ". "
                 } else {
                     "- ".to_string()
                 };

                 let (item_md, item_losses) = item.to_markdown(context);

                 losses.merge(item_losses);

                 item_md
                     .split('\n')
                     .enumerate()
                     .map(|(index, line)| {
                         if index == 0 {
                             [bullet.clone(), line.to_string()].concat()
                         } else if line.trim().is_empty() {
                             String::new()
                         } else {
                             ["  ", line].concat()
                         }
                     })
                     .join("\n")
             })
             .collect();

         // Keep lists tight if no items have internal newlines
         let mut tight = true;
         for item in &items {
             if item.trim().contains('\n') {
                 tight = false;
                 break;
             }
         }
         let items = items
             .iter()
             .map(|item| item.trim())
             .join(if tight { "\n" } else { "\n\n" });

         let md = [items.as_str(), "\n\n"].concat();
        */
    }
}
