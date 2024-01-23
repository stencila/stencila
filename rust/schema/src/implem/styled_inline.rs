use codec_losses::lost_options;

use crate::{prelude::*, StyledInline};

impl MarkdownCodec for StyledInline {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id))
            .merge_losses(lost_options!(
                self.options,
                compilation_digest,
                compilation_errors,
                css,
                classes
            ))
            .push_str("[")
            .push_prop_fn("content", |context| self.content.to_markdown(context))
            .push_str("]{")
            .push_prop_str("code", &self.code)
            .push_str("}");

        if let Some(lang) = &self.style_language {
            context
                .push_str("{")
                .push_prop_str("style_language", lang)
                .push_str("}");
        }

        context.exit_node();
    }
}
