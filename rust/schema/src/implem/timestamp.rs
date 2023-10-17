use crate::{prelude::*, Timestamp};

impl Timestamp {
    pub fn to_jats_special(&self) -> (String, Losses) {
        use codec_jats_trait::encode::elem;

        (
            elem(
                "timestamp",
                [("value", &self.value)],
                self.value.to_string(),
            ),
            Losses::none(),
        )
    }
}
