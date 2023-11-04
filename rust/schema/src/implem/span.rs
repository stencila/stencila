use codec_losses::lost_options;

use crate::{prelude::*, Span};

impl Span {
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

        let lang = self.style_language.as_deref().unwrap_or_default();

        let md = ["[", &md, "]", lang, "{", &self.code, "}"].concat();

        (md, losses)
    }
}
