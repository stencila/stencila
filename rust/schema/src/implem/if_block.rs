use stencila_codec_info::lost_exec_options;

use crate::{IfBlock, IfBlockClause, prelude::*};

impl LatexCodec for IfBlock {
    fn to_latex(&self, context: &mut LatexEncodeContext) {
        context.enter_node(self.node_type(), self.node_id());

        if context.render {
            // Render the first active clause only
            for clause in &self.clauses {
                if clause.is_active.unwrap_or_default() {
                    context.property_fn(NodeProperty::Content, |context| {
                        clause.content.to_latex(context)
                    });
                    break;
                }
            }
            context.exit_node();

            return;
        }

        if let (1, Some(IfBlockClause { code, content, .. })) =
            (self.clauses.len(), self.clauses.first())
        {
            context
                .environ_begin("if")
                .char('{')
                .property_str(NodeProperty::Code, code)
                .char('}')
                .newline()
                .property_fn(NodeProperty::Content, |context| content.to_latex(context))
                .trim_end()
                .newline()
                .environ_end("if");
        } else {
            context.environ_begin("ifblock").newline();
            for (index, IfBlockClause { code, content, .. }) in self.clauses.iter().enumerate() {
                let environ = if index == 0 {
                    "if"
                } else if code.is_empty() && index == self.clauses.len() - 1 {
                    "else"
                } else {
                    "elif"
                };

                context
                    .environ_begin(environ)
                    .char('{')
                    .property_str(NodeProperty::Code, code)
                    .char('}')
                    .newline()
                    .property_fn(NodeProperty::Content, |context| content.to_latex(context))
                    .trim_end()
                    .newline()
                    .environ_end(environ);
            }
            context.trim_end().newline().environ_end("ifblock");
        }

        context.exit_node().newline();
    }
}

impl MarkdownCodec for IfBlock {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_exec_options!(self));

        // If rendering, or format anything other than Stencila Markdown,
        // encode the first active clause only
        if context.render || !matches!(context.format, Format::Smd) {
            for clause in &self.clauses {
                if clause.is_active.unwrap_or_default() {
                    context.push_prop_fn(NodeProperty::Content, |context| {
                        clause.content.to_markdown(context)
                    });
                    break;
                }
            }

            context.exit_node();
            return;
        }

        for (index, clause @ IfBlockClause { code, .. }) in self.clauses.iter().enumerate() {
            let keyword = if index == 0 {
                " if "
            } else if code.is_empty() && index == self.clauses.len() - 1 {
                " else"
            } else {
                " elif "
            };

            context
                .push_colons()
                .push_str(keyword)
                .increase_depth()
                .push_prop_fn(NodeProperty::Clauses, |context| clause.to_markdown(context))
                .decrease_depth();
        }

        if !self.clauses.is_empty() {
            context.push_colons().newline().exit_node().newline();
        } else {
            context.exit_node();
        }
    }
}
