use codec_losses::lost_options;

use crate::{prelude::*, MathInline};

impl MathInline {
    pub fn to_jats_special(&self) -> (String, Losses) {
        use codec_jats_trait::encode::{elem, elem_no_attrs};

        let mathml = self
            .mathml
            .as_ref()
            .map(|mathml| elem_no_attrs("mml:math", mathml))
            .unwrap_or_default();

        let mut attrs = vec![("code", self.code.as_str())];
        if let Some(lang) = &self.math_language {
            attrs.push(("language", lang.as_str()));
        }

        let jats = elem("inline-formula", attrs, mathml);

        let losses = lost_options!(self, id, compilation_digest, compilation_errors);

        (jats, losses)
    }

    pub fn to_markdown_special(&self, _context: &mut MarkdownEncodeContext) -> (String, Losses) {
        let lang = self
            .math_language
            .as_deref()
            .unwrap_or("tex")
            .to_lowercase();

        let md = if lang == "tex" {
            ["$", &self.code.replace('$', r"\$"), "$"].concat()
        } else {
            ["`", &self.code.replace('`', r"\`"), "`{", &lang, "}"].concat()
        };

        let losses = lost_options!(self, id, compilation_digest, compilation_errors, mathml);

        (md, losses)
    }
}
