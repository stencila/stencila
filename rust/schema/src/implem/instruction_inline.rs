use codec_losses::lost_exec_options;

use crate::{prelude::*, InstructionInline};

impl InstructionInline {
    pub fn to_markdown_special(&self, context: &mut MarkdownEncodeContext) -> (String, Losses) {
        let mut losses = lost_exec_options!(self);

        let mut md = "{%% ".to_string();

        if let Some(assignee) = &self.assignee {
            md += "@";
            md += assignee;
            md += " ";
        }

        md += &self.text;
        md += " ";

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
