use codec_losses::Losses;
use codec_text_traits::ToText;

use crate::TextValue;

impl ToText for TextValue {
    fn to_text(&self) -> (String, Losses) {
        (self.0.to_string(), Losses::none())
    }
}
