use codec_losses::lost_exec_options;

use crate::{prelude::*, InstructionInline};

impl InstructionInline {
    pub fn to_markdown_special(&self, context: &mut MarkdownEncodeContext) -> (String, Losses) {
        let mut losses = lost_exec_options!(self);

        let md = if let Some(content) = &self.content {
            let (content_md, content_losses) = content.to_markdown(context);
            losses.merge(content_losses);

            ["{%%", &self.text, "%>", &content_md, "%%}"].concat()
        } else {
            ["{@@", &self.text, "@@}"].concat()
        };

        (md, losses)
    }
}
