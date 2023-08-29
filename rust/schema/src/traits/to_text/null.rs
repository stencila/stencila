use codec_losses::{Loss, LossDirection, Losses};
use codec_text_traits::ToText;

use crate::Null;

impl ToText for Null {
    fn to_text(&self) -> (String, Losses) {
        (
            self.to_string(),
            Losses::new([Loss::of_type(LossDirection::Encode, "Null")]),
        )
    }
}
