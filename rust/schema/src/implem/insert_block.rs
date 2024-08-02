use codec_info::lost_options;

use crate::{prelude::*, InsertBlock};

impl InsertBlock {
    pub fn to_jats_special(&self) -> (String, Losses) {
        let (content, mut losses) = self.content.to_jats();

        losses.add("InsertBlock@");

        (content, losses)
    }
}

impl MarkdownCodec for InsertBlock {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id))
            .push_colons()
            .push_str(" insert");

        if let Some(feedback) = &self.feedback {
            context
                .push_str(" ")
                .push_prop_str(NodeProperty::Feedback, feedback);
        }

        context
            .push_str("\n\n")
            .increase_depth()
            .push_prop_fn(NodeProperty::Content, |context| {
                self.content.to_markdown(context)
            })
            .decrease_depth()
            .push_colons()
            .newline()
            .exit_node()
            .newline();
    }
}
