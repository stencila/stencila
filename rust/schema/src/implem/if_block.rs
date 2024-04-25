use codec_info::lost_exec_options;

use crate::{prelude::*, IfBlock, IfBlockClause};

impl MarkdownCodec for IfBlock {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_exec_options!(self));

        for (index, clause @ IfBlockClause { code, .. }) in self.clauses.iter().enumerate() {
            context
                .push_semis()
                .push_str(if index == 0 {
                    " if "
                } else if code.is_empty() && index == self.clauses.len() - 1 {
                    " else "
                } else {
                    " elif "
                })
                .increase_depth()
                .push_prop_fn("clause", |context| clause.to_markdown(context))
                .decrease_depth();
        }

        if !self.clauses.is_empty() {
            context.push_semis().newline().exit_node().newline();
        } else {
            context.exit_node();
        }
    }
}
