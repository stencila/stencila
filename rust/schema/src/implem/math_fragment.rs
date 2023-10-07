use codec_losses::{lost_options, lost_props};

use crate::{prelude::*, MathFragment};

impl MathFragment {
    pub fn to_jats_special(&self) -> (String, Losses) {
        use codec_jats_trait::encode::elem_no_attrs;

        let mathml = self
            .mathml
            .as_ref()
            .map(|mathml| elem_no_attrs("mml:math", mathml))
            .unwrap_or_default();

        let jats = elem_no_attrs("inline-formula", mathml);

        let mut losses = lost_props!(self, "code", "math_language");
        losses.merge(lost_options!(self, compile_digest, errors));

        (jats, losses)
    }

    pub fn to_markdown_special(&self, _context: &MarkdownEncodeContext) -> (String, Losses) {
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

        let losses = lost_options!(self, id, compile_digest, errors, mathml);

        (md, losses)
    }
}
