use codec_losses::lost_exec_options;

use crate::{prelude::*, IfBlockClause};

impl MarkdownCodec for IfBlockClause {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_exec_options!(self))
            .push_prop_str("code", &self.code);

        if let Some(lang) = &self.programming_language {
            if !lang.is_empty() {
                context
                    .push_str(" {")
                    .push_prop_str("programming_language", lang)
                    .push_str("}");
            }
        }

        context
            .push_str("\n\n")
            .push_prop_fn("content", |context| self.content.to_markdown(context))
            .exit_node();
    }
}
