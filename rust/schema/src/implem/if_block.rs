use codec_info::lost_exec_options;

use crate::{prelude::*, IfBlock, IfBlockClause};

impl MarkdownCodec for IfBlock {
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
            for clause in &self.clauses {
                if let Some(messages) = &clause.options.execution_messages {
                    for message in messages {
                        context.add_message(
                            clause.node_type(),
                            clause.node_id(),
                            message.level.clone().into(),
                            message.message.to_string(),
                        );
                    }
                }
            }

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
