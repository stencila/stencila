use crate::{prelude::*, Include};

impl Include {
    pub fn to_markdown_special(&self) -> (String, Losses) {
        let md = ["/", &self.source].concat();

        (md, Losses::todo())
    }
}
