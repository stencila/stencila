use codec_losses::{lost_exec_options, lost_options};

use crate::{prelude::*, CallBlock};

impl MarkdownCodec for CallBlock {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id, content))
            .merge_losses(lost_exec_options!(self));

        context
            .push_str("<< ")
            .push_prop_str("source", &self.source)
            .push_str("(");

        for (index, arg) in self.arguments.iter().enumerate() {
            if index != 0 {
                context.push_str(", ");
            }
            arg.to_markdown(context);
        }

        context.push_str(")");

        if self.auto_exec.is_some() || self.media_type.is_some() || self.select.is_some() {
            context.push_str(" {");

            let mut prefix = "";
            if let Some(auto) = &self.auto_exec {
                context
                    .push_str("auto=")
                    .push_prop_str("auto_exec", &auto.to_string().to_lowercase());
                prefix = " ";
            }

            if let Some(media_type) = &self.media_type {
                context
                    .push_str(prefix)
                    .push_str("format=")
                    .push_prop_str("media_type", media_type);
                prefix = " ";
            }

            if let Some(select) = &self.select {
                context
                    .push_str(prefix)
                    .push_str("select=")
                    .push_prop_str("select", select);
            }

            context.push_str("}");
        }

        context.newline().exit_node().newline();
    }
}
