use codec_losses::lost_options;

use crate::{prelude::*, StyledInline};

impl StyledInline {
    pub fn to_markdown_special(&self, context: &mut MarkdownEncodeContext) -> (String, Losses) {
        let mut losses = lost_options!(
            self,
            id,
            compilation_digest,
            compilation_errors,
            css,
            classes
        );

        let (md, md_losses) = self.content.to_markdown(context);
        losses.merge(md_losses);

        let lang = self
            .style_language
            .as_ref()
            .map(|lang| ["{", lang, "}"].concat())
            .unwrap_or_default();

        let md = ["[", &md, "]{", &self.code, "}", &lang].concat();

        (md, losses)
    }
}
