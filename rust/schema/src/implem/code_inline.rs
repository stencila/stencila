use codec_info::lost_options;

use crate::{prelude::*, CodeInline};

impl MarkdownCodec for CodeInline {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id));

        context
            .push_str("`")
            .push_prop_str("code", &self.code)
            .push_str("`");

        if let Some(lang) = &self.programming_language {
            context
                .push_str("{")
                .push_prop_str("programming_language", &lang.replace('}', r"\}"))
                .push_str("}");
        }

        context.exit_node();
    }
}
