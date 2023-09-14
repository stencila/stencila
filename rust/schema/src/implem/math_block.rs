use crate::{prelude::*, MathBlock};

impl MathBlock {
    pub fn to_markdown_special(&self) -> (String, Losses) {
        let md = if self.math_language.to_lowercase() == "tex" {
            ["$$\n", &self.code, "\n$$\n\n"].concat()
        } else {
            ["```", &self.math_language, "\n", &self.code, "\n```\n\n"].concat()
        };

        (md, Losses::todo())
    }
}
