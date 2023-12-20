use codec_losses::lost_options;

use crate::{prelude::*, StyledBlock};

impl MarkdownCodec for StyledBlock {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(
                self,
                id,
                compilation_digest,
                compilation_errors,
                css,
                classes
            ));

        let fence = ":".repeat(3 + context.depth * 2);
        context.push_str(&fence);

        if let Some(lang) = &self.style_language {
            context.push_str(" ").push_prop_str("style_language", lang);
        }

        context
            .push_str("{")
            .push_prop_str("code", &self.code)
            .push_str("}\n\n")
            .increase_depth()
            .push_prop_fn("content", |context| self.content.to_markdown(context))
            .decrease_depth()
            .push_str(&fence)
            .push_str("\n")
            .exit_node()
            .push_str("\n");
    }
}
