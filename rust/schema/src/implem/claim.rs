use codec_info::lost_work_options;

use crate::{prelude::*, Claim};

impl MarkdownCodec for Claim {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_work_options!(self))
            .push_semis()
            .push_str(" ")
            .push_prop_str("claim_type", &self.claim_type.to_string().to_lowercase());

        if let Some(label) = &self.label {
            context.push_str(" ").push_prop_str("label", label);
        }

        context
            .push_str("\n\n")
            .increase_depth()
            .push_prop_fn("content", |context| self.content.to_markdown(context))
            .decrease_depth()
            .push_semis()
            .newline()
            .exit_node()
            .newline();
    }
}
