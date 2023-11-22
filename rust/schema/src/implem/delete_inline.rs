use crate::{prelude::*, DeleteInline};

impl DeleteInline {
    pub fn to_jats_special(&self) -> (String, Losses) {
        (String::new(), Losses::one("DeleteInline"))
    }
}
