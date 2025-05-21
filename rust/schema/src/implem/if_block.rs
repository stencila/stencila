use codec_info::lost_exec_options;

use crate::{prelude::*, IfBlock, IfBlockClause};

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
        if matches!(context.format, Format::Llmd) {
            // Encode content of the first active clause only
            for clause in &self.clauses {
                if clause.is_active == Some(true) {
                    clause.content.to_markdown(context);
                    return;
                }
            }

            return;
        }

        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_exec_options!(self));

        for (index, clause @ IfBlockClause { code, .. }) in self.clauses.iter().enumerate() {
            let keyword = if index == 0 {
                "if"
            } else if code.is_empty() && index == self.clauses.len() - 1 {
                "else"
            } else {
                "elif"
            };

            let start = if matches!(context.format, Format::Myst) {
                ["{", keyword, if keyword == "else" { "}" } else { "} " }].concat()
            } else {
                [" ", keyword, " "].concat()
            };

            context
                .push_colons()
                .push_str(&start)
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
