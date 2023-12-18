use codec_losses::lost_exec_options;

use crate::{prelude::*, InstructionBlock};

impl InstructionBlock {
    pub fn to_markdown_special(&self, context: &mut MarkdownEncodeContext) -> (String, Losses) {
        let mut losses = lost_exec_options!(self);

        let md = if let Some(content) = &self.content {
            let (content_md, content_losses) = content.to_markdown(context);
            losses.merge(content_losses);

            ["%% ", &self.text, "\n\n", &content_md, "%%\n\n"].concat()
        } else {
            ["@@ ", &self.text, "\n\n"].concat()
        };

        (md, losses)
    }
}
