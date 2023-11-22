use codec_losses::lost_options;

use crate::{prelude::*, DeleteBlock};

impl DeleteBlock {
    pub fn to_jats_special(&self) -> (String, Losses) {
        (String::new(), Losses::one("DeleteBlock"))
    }

    pub fn to_markdown_special(&self, context: &mut MarkdownEncodeContext) -> (String, Losses) {
        let mut losses = lost_options!(self, id);

        let fence = ":".repeat(3 + context.depth * 2);

        context.down();
        let (md, md_losses) = self.content.to_markdown(context);
        context.up();

        losses.merge(md_losses);

        let md = [&fence, " delete\n\n", &md, &fence, "\n\n"].concat();

        (md, losses)
    }
}
