use crate::{prelude::*, StyledInline};

impl MarkdownCodec for StyledInline {
    fn to_markdown(&self, _context: &mut MarkdownEncodeContext) {
        /*
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
        */
    }
}
