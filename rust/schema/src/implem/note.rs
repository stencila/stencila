use codec_losses::lost_options;

use crate::{prelude::*, Note};

impl Note {
    pub fn to_markdown_special(&self, context: &mut MarkdownEncodeContext) -> (String, Losses) {
        let mut losses = lost_options!(self, id);

        let (content, content_losses) = self.content.to_markdown(context);
        losses.merge(content_losses);

        // This content is added to the Markdown by `Article::to_markdown_special`
        context.footnotes.push(content);
        let index = context.footnotes.len();

        let md = ["[^", &index.to_string(), "]"].concat();

        (md, losses)
    }
}
