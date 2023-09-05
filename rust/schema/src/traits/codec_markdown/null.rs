use crate::Null;

use super::prelude::*;

impl MarkdownCodec for Null {
    fn to_markdown(&self) -> (String, Losses) {
        (
            self.to_string(),
            Losses::new([Loss::of_type(LossDirection::Encode, "Null")]),
        )
    }
}
