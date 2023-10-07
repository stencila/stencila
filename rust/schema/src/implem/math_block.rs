use codec_losses::{lost_options, lost_props};

use crate::{prelude::*, MathBlock};

impl MathBlock {
    pub fn to_jats_special(&self) -> (String, Losses) {
        use codec_jats_trait::encode::elem_no_attrs;

        let label = self
            .label
            .as_ref()
            .map(|label| elem_no_attrs("label", label))
            .unwrap_or_default();

        let mathml = self
            .mathml
            .as_ref()
            .map(|mathml| elem_no_attrs("mml:math", mathml))
            .unwrap_or_default();

        let jats = elem_no_attrs("disp-formula", [label, mathml].concat());

        let mut losses = lost_props!(self, "code", "math_language");
        losses.merge(lost_options!(self, id, compile_digest, errors));

        (jats, losses)
    }

    pub fn to_markdown_special(&self, _context: &MarkdownEncodeContext) -> (String, Losses) {
        let md = if self.math_language.to_lowercase() == "tex" {
            ["$$\n", &self.code, "\n$$\n\n"].concat()
        } else {
            ["```", &self.math_language, "\n", &self.code, "\n```\n\n"].concat()
        };

        let losses = lost_options!(self, id, compile_digest, errors, mathml, label);

        (md, losses)
    }
}
