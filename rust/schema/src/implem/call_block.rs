use codec_losses::{lost_exec_options, lost_options};

use crate::{prelude::*, CallBlock};

impl CallBlock {
    pub fn to_markdown_special(&self, _context: &mut MarkdownEncodeContext) -> (String, Losses) {
        let mut md = ["/", &self.source, "("].concat();

        for (index, arg) in self.arguments.iter().enumerate() {
            if index != 0 {
                md.push_str(", ");
            }
            md.push_str(&arg.name);

            md.push('=');

            if arg.code.contains([',', ' ', ')']) {
                md.push('`');
                md.push_str(&arg.code);
                md.push('`');
            } else {
                md.push_str(&arg.code);
            }
        }

        md.push_str(")\n\n");

        let mut losses = lost_options!(self, id, media_type, select, content);
        losses.merge(lost_exec_options!(self));

        (md, losses)
    }
}
