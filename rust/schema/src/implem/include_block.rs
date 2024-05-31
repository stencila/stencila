use codec_info::{lost_exec_options, lost_options};

use crate::{prelude::*, IncludeBlock};

impl MarkdownCodec for IncludeBlock {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        if context.render {
            // Encode content only
            if let Some(content) = &self.content {
                content.to_markdown(context);
            }
            return;
        }

        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id))
            .merge_losses(lost_exec_options!(self))
            .push_semis()
            .push_str(" include ")
            .push_prop_str(NodeProperty::Source, &self.source);

        if self.auto_exec.is_some() || self.media_type.is_some() || self.select.is_some() {
            context.push_str(" {");

            let mut prefix = "";
            if let Some(auto) = &self.auto_exec {
                context
                    .push_str("auto=")
                    .push_prop_str(NodeProperty::AutoExec, &auto.to_string().to_lowercase());
                prefix = " ";
            }

            if let Some(media_type) = &self.media_type {
                context
                    .push_str(prefix)
                    .push_str("format=")
                    .push_prop_str(NodeProperty::MediaType, media_type);
                prefix = " ";
            }

            if let Some(select) = &self.select {
                context
                    .push_str(prefix)
                    .push_str("select=")
                    .push_prop_str(NodeProperty::Select, select);
            }

            context.push_str("}");
        }

        context.newline().exit_node().newline();
    }
}
