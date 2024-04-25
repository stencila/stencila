use codec_info::lost_options;

use crate::{prelude::*, CodeBlock};

impl MarkdownCodec for CodeBlock {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id));

        context.push_str("```");

        if let Some(lang) = &self.programming_language {
            context.push_prop_str(NodeProperty::ProgrammingLanguage, lang);
        }

        context
            .newline()
            .push_prop_str(NodeProperty::Code, &self.code);

        if !self.code.ends_with('\n') {
            context.newline();
        }

        context.push_str("```\n").exit_node().newline();
    }
}
