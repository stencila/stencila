use crate::{prelude::*, Delete};

impl Delete {
    pub fn to_jats_special(&self) -> (String, Losses) {
        (String::new(), Losses::one("Delete"))
    }
}
