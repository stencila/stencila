use codec_losses::lost_exec_options;

use crate::{prelude::*, IfBlock, IfBlockClause};

impl MarkdownCodec for IfBlock {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_exec_options!(self));

        let fence = ":".repeat(3 + context.depth * 2);

        for (index, clause @ IfBlockClause { code, .. }) in self.clauses.iter().enumerate() {
            context
                .push_str(&fence)
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
            context
                .push_str(&fence)
                .push_str("\n")
                .exit_node()
                .push_str("\n");
        }

        context.exit_node();
    }
}
