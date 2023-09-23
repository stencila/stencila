use crate::{prelude::*, MathBlock};

impl MathBlock {
    pub fn to_jats_special(&self) -> (String, Losses) {
        use codec_jats_trait::encode::elem;

        let label = self
            .label.as_ref()
            .map(|label| elem("label", [], label))
            .unwrap_or_default();
        
        let mathml = self
            .mathml.as_ref()
            .map(|mathml| elem("mml:math", [], mathml))
            .unwrap_or_default();

        (elem("disp-formula", [], [label, mathml].concat()), Losses::todo())
    }

    pub fn to_markdown_special(&self) -> (String, Losses) {
        let md = if self.math_language.to_lowercase() == "tex" {
            ["$$\n", &self.code, "\n$$\n\n"].concat()
        } else {
            ["```", &self.math_language, "\n", &self.code, "\n```\n\n"].concat()
        };

        (md, Losses::todo())
    }
}
