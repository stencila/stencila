use codec_losses::{lost_exec_options, lost_options};

use crate::{prelude::*, CodeChunk};

impl CodeChunk {
    pub fn to_markdown_special(&self, _context: &mut MarkdownEncodeContext) -> (String, Losses) {
        let mut losses = lost_options!(self, id, outputs);
        losses.merge(lost_exec_options!(self));

        let mut md = "```".to_string();

        if let Some(lang) = &self.programming_language {
            md.push_str(lang);
            md.push(' ');
        }

        md.push_str("exec");

        if let Some(auto) = &self.auto_exec {
            md.push_str(" auto=");
            md.push_str(&auto.to_string().to_lowercase())
        }

        md.push('\n');
        md.push_str(&self.code);

        if !self.code.ends_with('\n') {
            md.push('\n');
        }

        md.push_str("```\n\n");

        (md, losses)
    }
}
