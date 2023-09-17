use crate::{prelude::*, Text};

impl Text {
    pub fn to_jats_special(&self) -> (String, Losses) {
        use codec_jats_trait::encode::escape;

        (escape(&self.value.0).to_string(), Losses::none())
    }
}
