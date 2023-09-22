use crate::{prelude::*, Insert};

impl Insert {
    pub fn to_jats_special(&self) -> (String, Losses) {
        let (content, mut losses) = self.content.to_jats();

        losses.add(Loss::of_type(LossDirection::Encode, "Insert"));
        losses.add(Loss::of_property(
            LossDirection::Encode,
            "Insert",
            "suggester",
        ));

        (content, losses)
    }
}
