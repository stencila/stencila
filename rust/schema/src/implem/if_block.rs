use codec_info::lost_exec_options;

use crate::{prelude::*, IfBlock, IfBlockClause};

impl MarkdownCodec for IfBlock {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        if context.render {
            // Encode content of the first active clause only
            for clause in self.clauses.iter() {
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
                .push_semis()
                .push_str(&start)
                .increase_depth()
                .push_prop_fn(NodeProperty::Clauses, |context| clause.to_markdown(context))
                .decrease_depth();
        }

        if !self.clauses.is_empty() {
            context.push_semis().newline().exit_node().newline();
        } else {
            context.exit_node();
        }
    }
}
