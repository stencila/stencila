use codec_losses::lost_options;

use crate::{prelude::*, QuoteBlock};

impl QuoteBlock {
    pub fn to_markdown_special(&self, context: &MarkdownEncodeContext) -> (String, Losses) {
        let mut losses = lost_options!(self, id, cite);

        let (content, content_losses) = self.content.to_markdown(context);
        losses.merge(content_losses);

        let content = content
            .trim()
            .lines()
            .map(|line| ["> ", line].concat())
            .join("\n");

        let md = [content, "\n\n".to_string()].concat();

        (md, losses)
    }
}
