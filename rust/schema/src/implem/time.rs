use crate::{prelude::*, Time};

impl Time {
    pub fn to_jats_special(&self) -> (String, Losses) {
        use codec_jats_trait::encode::elem;

        (
            elem("time", [("iso-8601-time", &self.value)], &self.value),
            Losses::none(),
        )
    }
}
