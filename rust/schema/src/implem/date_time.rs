use crate::{prelude::*, DateTime};

impl DateTime {
    pub fn to_jats_special(&self) -> (String, Losses) {
        use codec_jats_trait::encode::elem;

        (
            elem(
                "date-time",
                [("iso-8601-date-time", &self.value)],
                &self.value,
            ),
            Losses::none(),
        )
    }
}
