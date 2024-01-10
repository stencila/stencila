use codec_losses::lost_exec_options;

use crate::{prelude::*, InstructionInline};

impl InstructionInline {
    pub fn to_markdown_special(&self, context: &mut MarkdownEncodeContext) -> (String, Losses) {
        let mut losses = lost_exec_options!(self);

        let mut md = "{%% ".to_string();

        if let Some(assignee) = &self.options.assignee {
            md += "@";
            md += assignee;
            md += " ";
        }

        if let Some(part) = self
            .messages
            .first()
            .and_then(|message| message.parts.first())
        {
            let (part_md, part_losses) = part.to_markdown(context);
            losses.merge(part_losses);

            md += &part_md;
            md += " ";
        }

        if let Some(content) = &self.content {
            let (content_md, content_losses) = content.to_markdown(context);
            losses.merge(content_losses);

            md += "%>";
            md += &content_md;
        };

        md += "%%}";

        (md, losses)
    }
}
