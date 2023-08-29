use crate::Array;

use super::prelude::*;

impl TextCodec for Array {
    fn to_text(&self) -> (String, Losses) {
        let mut text = String::new();
        let mut losses = Losses::none();

        for (index, item) in self.iter().enumerate() {
            if index != 0 {
                text.push(' ');
            }

            let (item_text, mut item_losses) = item.to_text();
            text.push_str(&item_text);
            losses.add_all(&mut item_losses);
        }

        (text, losses)
    }
}
