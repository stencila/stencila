use codec_losses::{lost_exec_options, lost_options};

use crate::{prelude::*, CallBlock};

impl MarkdownCodec for CallBlock {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id, media_type, select, content))
            .merge_losses(lost_exec_options!(self));

        context
            .push_str("/")
            .push_prop_str("source", &self.source)
            .push_str("(");

        for (index, arg) in self.arguments.iter().enumerate() {
            if index != 0 {
                context.push_str(", ");
            }
            arg.to_markdown(context);
        }

        context.push_str(")\n").exit_node().push_str("\n");
    }
}
