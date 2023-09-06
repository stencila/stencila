use crate::{prelude::*, MathFragment};

impl MathFragment {
    pub fn to_markdown_special(&self) -> (String, Losses) {
        let md = if self.math_language.to_lowercase() == "tex" {
            ["$", &self.code.replace('$', r"\$"), "$"].concat()
        } else {
            [
                "`",
                &self.code.replace('`', r"\`"),
                "`{",
                &self.math_language.replace('}', r"\}"),
                "}",
            ]
            .concat()
        };

        (md, Losses::todo())
    }
}
