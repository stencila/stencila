use codec_losses::{lost_exec_options, lost_options};

use crate::{prelude::*, CodeExpression};

impl MarkdownCodec for CodeExpression {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id, output))
            .merge_losses(lost_exec_options!(self));

        context
            .push_str("`")
            .push_prop_str("code", &self.code)
            .push_str("`{");

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

        context.push_str("}").exit_node();
    }
}
