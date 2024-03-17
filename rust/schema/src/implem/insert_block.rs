use codec_losses::lost_options;

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
            .push_str("++");

        if let Some(status) = &self.suggestion_status {
            context
                .push_str(" ")
                .push_prop_str("suggestion_status", &status.to_string().to_lowercase());
        }

        context
            .push_str("\n\n")
            .push_prop_fn("content", |context| self.content.to_markdown(context))
            .push_str("++\n")
            .exit_node()
            .newline();
    }
}
