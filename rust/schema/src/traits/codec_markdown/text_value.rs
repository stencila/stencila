use crate::TextValue;

use super::prelude::*;

impl MarkdownCodec for TextValue {
    fn to_markdown(&self) -> (String, Losses) {
        (self.0.to_string(), Losses::none())
    }
}
