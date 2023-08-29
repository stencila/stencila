use codec_losses::{Loss, LossDirection, Losses};
use codec_text_traits::ToText;

use crate::Object;

impl ToText for Object {
    fn to_text(&self) -> (String, Losses) {
        let mut text = String::new();
        let mut losses = Losses::new([Loss::of_structure(LossDirection::Encode, "Object")]);

        for (name, value) in self.iter() {
            if !text.is_empty() {
                text.push(' ');
            }

            text.push_str(name);

            text.push(' ');

            let (value_text, mut value_losses) = value.to_text();
            text.push_str(&value_text);
            losses.add_all(&mut value_losses);
        }

        (text, losses)
    }
}
