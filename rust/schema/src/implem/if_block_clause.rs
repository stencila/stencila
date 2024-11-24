use codec_info::lost_exec_options;

use crate::{prelude::*, IfBlockClause};

impl MarkdownCodec for IfBlockClause {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_exec_options!(self))
            .push_prop_fn(NodeProperty::Code, |context| self.code.to_markdown(context));

        if matches!(context.format, Format::Markdown | Format::Smd | Format::Qmd) {
            if let Some(lang) = &self.programming_language {
                if !lang.is_empty() {
                    context
                        .push_str(" {")
                        .push_prop_str(NodeProperty::ProgrammingLanguage, lang)
                        .push_str("}");
                }
            }
        }

        context
            .push_str("\n\n")
            .push_prop_fn(NodeProperty::Content, |context| {
                self.content.to_markdown(context)
            })
            .exit_node();
    }
}
