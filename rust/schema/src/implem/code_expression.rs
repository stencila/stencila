use codec_info::{lost_exec_options, lost_options};

use crate::{prelude::*, CodeExpression};

impl MarkdownCodec for CodeExpression {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        if context.render {
            // Record any execution messages
            if let Some(messages) = &self.options.execution_messages {
                for message in messages {
                    context.add_message(
                        self.node_type(),
                        self.node_id(),
                        message.level.clone().into(),
                        message.message.to_string(),
                    );
                }
            }

            // Encode output only
            if let Some(output) = &self.output {
                output.to_markdown(context);
            }

            return;
        }

        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id, output))
            .merge_losses(lost_exec_options!(self));

        if matches!(context.format, Format::Myst) {
            context
                .merge_losses(lost_options!(self, programming_language, execution_mode))
                .myst_role("eval", |context| {
                    context
                        .push_prop_fn(NodeProperty::Code, |context| self.code.to_markdown(context));
                });
        } else {
            context
                .push_str("`")
                .push_prop_fn(NodeProperty::Code, |context| self.code.to_markdown(context))
                .push_str("`{");

            if let Some(lang) = &self.programming_language {
                if !lang.is_empty() {
                    context
                        .push_prop_str(NodeProperty::ProgrammingLanguage, lang)
                        .push_str(" ");
                }
            }

            context.push_str("exec");

            if let Some(mode) = &self.execution_mode {
                context.push_str(" ").push_prop_str(
                    NodeProperty::ExecutionMode,
                    &mode.to_string().to_lowercase(),
                );
            }

            context.push_str("}");
        }

        context.exit_node();
    }
}
