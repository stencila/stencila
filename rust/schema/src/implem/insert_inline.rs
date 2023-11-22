use crate::{prelude::*, InsertInline};

impl InsertInline {
    pub fn to_jats_special(&self) -> (String, Losses) {
        let (content, mut losses) = self.content.to_jats();

        losses.add("InsertInline@");

        (content, losses)
    }
}
