use crate::{prelude::*, CodeBlock};

impl CodeBlock {
    pub fn to_markdown_special(&self) -> (String, Losses) {
        let mut md = "```".to_string();

        if let Some(lang) = &self.programming_language {
            md.push_str(lang);
        }

        md.push('\n');
        md.push_str(&self.code.0);

        if !self.code.ends_with('\n') {
            md.push('\n');
        }

        md.push_str("```\n\n");

        let losses = if self.id.is_some() {
            Losses::one("CodeBlock.id")
        } else {
            Losses::none()
        };

        (md, losses)
    }
}
