use crate::{prelude::*, Text};

impl Text {
    pub fn to_jats_special(&self) -> (String, Losses) {
        (self.value.to_string(), Losses::none())
    }
}
