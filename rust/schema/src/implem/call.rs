use codec_json5_trait::Json5Codec;
use codec_losses::{lost_exec_options, lost_options};

use crate::{prelude::*, Call};

impl Call {
    pub fn to_markdown_special(&self, _context: &MarkdownEncodeContext) -> (String, Losses) {
        let mut md = ["/", &self.source, "("].concat();

        for arg in &self.arguments {
            md.push_str(&arg.name);
            md.push('=');
            md.push_str(&arg.code.to_json5().unwrap_or_default());
        }

        md.push_str(")\n\n");

        let mut losses = lost_options!(self, id, media_type, select, content);
        losses.merge(lost_exec_options!(self));

        (md, losses)
    }
}
