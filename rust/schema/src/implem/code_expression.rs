use codec_losses::{lost_exec_options, lost_options};

use crate::{prelude::*, CodeExpression};

impl CodeExpression {
    pub fn to_markdown_special(&self, _context: &mut MarkdownEncodeContext) -> (String, Losses) {
        let mut losses = lost_options!(self, id, output);
        losses.merge(lost_exec_options!(self));

        let mut md = ["`", &self.code, "`{"].concat();

        if let Some(lang) = &self.programming_language {
            md.push_str(lang);
            md.push(' ');
        }

        md.push_str("exec");

        if let Some(auto) = &self.auto_exec {
            md.push_str(" auto=");
            md.push_str(&auto.to_string().to_lowercase())
        }

        md.push('}');

        (md, losses)
    }
}
