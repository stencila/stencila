use crate::{prelude::*, Date};

impl Date {
    pub fn to_jats_special(&self) -> (String, Losses) {
        use codec_jats_trait::encode::elem;

        (
            elem("date", [("iso-8601-date", &self.value)], &self.value),
            Losses::none(),
        )
    }
}
