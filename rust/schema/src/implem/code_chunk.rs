use codec_losses::{lost_exec_options, lost_options};

use crate::{prelude::*, CodeChunk};

impl MarkdownCodec for CodeChunk {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id, outputs))
            .merge_losses(lost_exec_options!(self));

        context.push_str("```");

        if let Some(lang) = &self.programming_language {
            context
                .push_prop_str("programming_language", lang)
                .push_str(" ");
        }

        context.push_str("exec");

        if let Some(auto) = &self.auto_exec {
            context
                .push_str(" auto=")
                .push_prop_str("auto_exec", &auto.to_string().to_lowercase());
        }

        context.push_str("\n").push_prop_str("code", &self.code);

        if !self.code.ends_with('\n') {
            context.push_str("\n");
        }

        context.push_str("```\n").exit_node().push_str("\n");
    }
}
