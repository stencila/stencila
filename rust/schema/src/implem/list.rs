use codec_html_trait::encode::elem;

use crate::{prelude::*, List, ListOrder};

impl List {
    pub fn to_html_special(&self) -> String {
        let tag = match &self.order {
            ListOrder::Ascending => "ol",
            _ => "ul",
        };

        let items = self.items.to_html();

        elem(tag, &[], &[items])
    }

    pub fn to_markdown_special(&self, context: &mut MarkdownEncodeContext) -> (String, Losses) {
        let mut losses = Losses::none();

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

        (md, losses)
    }
}
