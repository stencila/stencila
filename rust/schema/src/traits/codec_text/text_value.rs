use crate::TextValue;

use super::prelude::*;

impl TextCodec for TextValue {
    fn to_text(&self) -> (String, Losses) {
        (self.0.to_string(), Losses::none())
    }
}
