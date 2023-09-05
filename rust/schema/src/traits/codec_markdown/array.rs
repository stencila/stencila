use crate::Array;

use super::prelude::*;

impl MarkdownCodec for Array {
    fn to_markdown(&self) -> (String, Losses) {
        let mut markdown = String::new();
        let mut losses = Losses::none();

        for (index, item) in self.iter().enumerate() {
            if index != 0 {
                markdown.push(' ');
            }

            let (item_markdown, mut item_losses) = item.to_markdown();
            markdown.push_str(&item_markdown);
            losses.add_all(&mut item_losses);
        }

        (markdown, losses)
    }
}
