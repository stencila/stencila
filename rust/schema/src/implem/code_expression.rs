use codec_json5_trait::Json5Codec;
use codec_losses::{lost_exec_options, lost_options};

use crate::{prelude::*, CodeExpression};

impl CodeExpression {
    pub fn to_markdown_special(&self, _context: &MarkdownEncodeContext) -> (String, Losses) {
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

        if let Some(output) = self
            .output
            .as_ref()
            .and_then(|output| output.to_json5().ok())
        {
            md.push_str(" output=");
            md.push_str(&output);
        }

        md.push('}');

        let mut losses = lost_options!(self, id);
        losses.merge(lost_exec_options!(self));

        (md, losses)
    }
}
