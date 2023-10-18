use crate::{prelude::*, Duration};

impl Duration {
    pub fn to_jats_special(&self) -> (String, Losses) {
        use codec_jats_trait::encode::elem;

        (
            elem(
                "duration",
                [
                    ("value", self.value.to_string()),
                    ("unit", self.time_unit.to_string()),
                ],
                format!("{} {}s", self.value, self.time_unit),
            ),
            Losses::none(),
        )
    }
}
