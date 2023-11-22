use codec_losses::lost_options;

use crate::{prelude::*, InsertBlock};

impl InsertBlock {
    pub fn to_jats_special(&self) -> (String, Losses) {
        let (content, mut losses) = self.content.to_jats();

        losses.add("InsertBlock@");

        (content, losses)
    }

    pub fn to_markdown_special(&self, context: &mut MarkdownEncodeContext) -> (String, Losses) {
        let mut losses = lost_options!(self, id);

        let fence = ":".repeat(3 + context.depth * 2);

        context.down();
        let (md, md_losses) = self.content.to_markdown(context);
        context.up();

        losses.merge(md_losses);

        let md = [&fence, " insert\n\n", &md, &fence, "\n\n"].concat();

        (md, losses)
    }
}
