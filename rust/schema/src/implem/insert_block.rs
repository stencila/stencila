use crate::{prelude::*, InsertBlock};

impl InsertBlock {
    pub fn to_jats_special(&self) -> (String, Losses) {
        let (content, mut losses) = self.content.to_jats();

        losses.add("InsertBlock@");

        (content, losses)
    }
}
