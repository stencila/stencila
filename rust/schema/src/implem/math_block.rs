use codec_losses::lost_options;

use crate::{prelude::*, MathBlock};

impl MathBlock {
    pub fn to_jats_special(&self) -> (String, Losses) {
        use codec_jats_trait::encode::{elem, elem_no_attrs};

        let label = self
            .label
            .as_ref()
            .map(|label| elem_no_attrs("label", label))
            .unwrap_or_default();

        let mathml = self
            .options
            .mathml
            .as_ref()
            .map(|mathml| elem_no_attrs("mml:math", mathml))
            .unwrap_or_default();

        let mut attrs = vec![("code", self.code.as_str())];
        if let Some(lang) = &self.math_language {
            attrs.push(("language", lang.as_str()));
        }

        let jats = elem("disp-formula", attrs, [label, mathml].concat());

        let mut losses = lost_options!(self, id);
        losses.merge(lost_options!(
            self.options,
            compilation_digest,
            compilation_errors
        ));

        (jats, losses)
    }

    pub fn to_markdown_special(&self, _context: &mut MarkdownEncodeContext) -> (String, Losses) {
        let lang = self
            .math_language
            .as_deref()
            .unwrap_or("tex")
            .to_lowercase();

        let mut code = self.code.clone();
        if !code.ends_with('\n') {
            code.push('\n');
        }

        let md = if lang == "tex" {
            ["$$\n", &code, "$$\n\n"].concat()
        } else {
            ["```", &lang, "\n", &code, "```\n\n"].concat()
        };

        let mut losses = lost_options!(self, id, label);
        losses.merge(lost_options!(
            self.options,
            compilation_digest,
            compilation_errors,
            mathml
        ));

        (md, losses)
    }
}
