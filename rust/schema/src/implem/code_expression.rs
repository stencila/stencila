use codec_info::{lost_exec_options, lost_options};

use crate::{prelude::*, CodeExpression};

impl LatexCodec for CodeExpression {
    fn to_latex(&self, context: &mut LatexEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id, execution_mode, execution_bounds));

        if matches!(context.format, Format::Rnw) {
            context
                .str(r"\Sexpr{")
                .property_fn(NodeProperty::Code, |context| self.code.to_latex(context))
                .str("}");
        } else if let Some(output) = &self.output {
            context
                .add_loss("CodeExpression.code")
                .property_fn(NodeProperty::Output, |context| output.to_latex(context));
        } else {
            context.property_str(NodeProperty::Code, &self.code);
        }

        context.exit_node();
    }
}

impl MarkdownCodec for CodeExpression {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id, output))
            .merge_losses(lost_exec_options!(self));

        if matches!(context.format, Format::Myst) {
            context
                .merge_losses(lost_options!(self, programming_language, execution_mode))
                .myst_role("eval", |context| {
                    context.push_prop_str(NodeProperty::Code, &self.code);
                });
        } else if matches!(context.format, Format::Qmd) {
            context.push_str("`");

            if let Some(lang) = &self.programming_language {
                context
                    .push_str("{")
                    .push_prop_str(NodeProperty::ProgrammingLanguage, lang)
                    .push_str("} ");
            } else {
                context.push_str("{}");
            }

            context
                .push_prop_str(NodeProperty::Code, &self.code)
                .push_str("`");
        } else if matches!(self.programming_language.as_deref(), Some("jinja")) {
            context
                .push_str("{{ ")
                .push_prop_str(NodeProperty::Code, &self.code)
                .push_str(" }}");
        } else {
            context
                .push_str("`")
                .push_prop_str(NodeProperty::Code, &self.code)
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

            if let Some(bounds) = &self.execution_bounds {
                context.push_str(" ").push_prop_str(
                    NodeProperty::ExecutionBounds,
                    &bounds.to_string().to_lowercase(),
                );
            }

            context.push_str("}");
        }

        if let (Format::Llmd, Some(output)) = (&context.format, &self.output) {
            context
                .push_str(" => ")
                .push_prop_fn(NodeProperty::Output, |context| output.to_markdown(context));
        }

        context.exit_node();
    }
}
