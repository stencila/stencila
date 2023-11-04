use codec_losses::lost_options;

use crate::{prelude::*, Link};

impl Link {
    pub fn to_markdown_special(&self, context: &mut MarkdownEncodeContext) -> (String, Losses) {
        let mut losses = lost_options!(self, id, rel);

        let (content_md, content_losses) = self.content.to_markdown(context);
        losses.merge(content_losses);

        let mut md = ["[", &content_md, "](", &self.target].concat();

        if let Some(title) = &self.title {
            let (title_text, title_losses) = title.to_text();
            losses.merge(title_losses);

            md.push_str(" \"");
            md.push_str(&title_text);
            md.push('"');
        }

        md.push(')');

        (md, losses)
    }
}
