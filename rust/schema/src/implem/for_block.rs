use codec_info::lost_exec_options;

use crate::{prelude::*, Block, ForBlock, Section};

impl MarkdownCodec for ForBlock {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        if context.render {
            // Encode iterations only (unwrapping the `Section` representing each as is
            // usually the case) but if none, render any `otherwise`
            for iteration in self.iterations.iter().flatten() {
                if let Block::Section(Section { content, .. }) = iteration {
                    content.to_markdown(context);
                } else {
                    iteration.to_markdown(context);
                }
            }
            if let (false, Some(otherwise)) = (
                self.iterations
                    .as_ref()
                    .map(|iterations| !iterations.is_empty())
                    .unwrap_or_default(),
                &self.otherwise,
            ) {
                otherwise.to_markdown(context)
            }
            return;
        }

        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_exec_options!(self));

        let (for_, else_) = if matches!(context.format, Format::Myst) {
            ("{for} ", "{else}")
        } else {
            (" for ", " else")
        };

        context
            .push_colons()
            .push_str(for_)
            .push_prop_str(NodeProperty::Variable, &self.variable)
            .push_str(" in ")
            .push_prop_fn(NodeProperty::Code, |context| self.code.to_markdown(context));

        if matches!(context.format, Format::Markdown) {
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
            .increase_depth()
            .push_prop_fn(NodeProperty::Content, |context| {
                self.content.to_markdown(context)
            })
            .decrease_depth();

        if let Some(otherwise) = &self.otherwise {
            context
                .push_colons()
                .push_str(else_)
                .push_str("\n\n")
                .increase_depth()
                .push_prop_fn(NodeProperty::Otherwise, |context| {
                    otherwise.to_markdown(context)
                })
                .decrease_depth();
        }

        context.push_colons().newline().exit_node().newline();
    }
}
