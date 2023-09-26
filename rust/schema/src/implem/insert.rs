use crate::{prelude::*, Insert};

impl Insert {
    pub fn to_jats_special(&self) -> (String, Losses) {
        let (content, mut losses) = self.content.to_jats();

        losses.add("Insert@");

        (content, losses)
    }
}
