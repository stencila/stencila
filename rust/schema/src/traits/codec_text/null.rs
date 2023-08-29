use crate::Null;

use super::prelude::*;

impl TextCodec for Null {
    fn to_text(&self) -> (String, Losses) {
        (
            self.to_string(),
            Losses::new([Loss::of_type(LossDirection::Encode, "Null")]),
        )
    }
}
