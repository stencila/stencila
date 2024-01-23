use codec_losses::{lost_exec_options, lost_options};

use crate::{prelude::*, CallArgument};

impl MarkdownCodec for CallArgument {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id))
            .merge_losses(lost_exec_options!(self));

        context.push_prop_str("name", &self.name).push_str("=");

        if self.code.contains([',', ' ', ')']) {
            context.push_str("`");
            context.push_prop_str("code", &self.code);
            context.push_str("`");
        } else {
            context.push_str(&self.code);
        }

        context.exit_node();
    }
}
