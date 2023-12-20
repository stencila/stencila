use codec_losses::lost_exec_options;

use crate::{prelude::*, InstructionInline};

impl MarkdownCodec for InstructionInline {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_exec_options!(self));

        if let Some(content) = &self.content {
            context
                .push_str("{%%")
                .push_prop_str("text", &self.text)
                .push_str("%>")
                .push_prop_fn("content", |context| content.to_markdown(context))
                .push_str("%%}");
        } else {
            context
                .push_str("{@@")
                .push_prop_str("text", &self.text)
                .push_str("@@}");
        }

        context.exit_node();
    }
}
