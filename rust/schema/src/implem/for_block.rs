use codec_info::lost_exec_options;

use crate::{prelude::*, ForBlock};

impl MarkdownCodec for ForBlock {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_exec_options!(self))
            .push_semis()
            .push_str(" for ")
            .push_prop_str(NodeProperty::Variable, &self.variable)
            .push_str(" in ")
            .push_prop_fn(NodeProperty::Code, |context| self.code.to_markdown(context));

        if let Some(lang) = &self.programming_language {
            if !lang.is_empty() {
                context
                    .push_str(" {")
                    .push_prop_str(NodeProperty::ProgrammingLanguage, lang)
                    .push_str("}");
            }
        }

        context
            .push_str("\n\n")
            .increase_depth()
            .push_prop_fn(NodeProperty::Content, |context| {
                self.content.to_markdown(context)
            })
            .decrease_depth();

        if let Some(otherwise) = &self.otherwise {
            context
                .push_semis()
                .push_str(" else\n\n")
                .increase_depth()
                .push_prop_fn(NodeProperty::Otherwise, |context| {
                    otherwise.to_markdown(context)
                })
                .decrease_depth();
        }

        context.push_semis().newline().exit_node().newline();
    }
}
