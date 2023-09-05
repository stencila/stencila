use crate::{prelude::*, CodeFragment};

impl CodeFragment {
    pub fn to_markdown_special(&self) -> (String, Losses) {
        let mut md = ["`", &self.code, "`"].concat();

        if let Some(lang) = &self.programming_language {
            md.push('{');
            md.push_str(lang);
            md.push('}');
        }

        let losses = if self.id.is_some() {
            Losses::of_id("CodeFragment")
        } else {
            Losses::none()
        };

        (md, losses)
    }
}
