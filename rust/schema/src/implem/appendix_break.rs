use crate::{AppendixBreak, prelude::*};

impl LatexCodec for AppendixBreak {
    fn to_latex(&self, context: &mut LatexEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .str("\\appendix\n\n")
            .exit_node();
    }
}

impl MarkdownCodec for AppendixBreak {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        if context.render || !matches!(context.format, Format::Smd) {
            return;
        }

        context
            .enter_node(self.node_type(), self.node_id())
            .push_colons()
            .push_str(" appendix\n\n")
            .exit_node();
    }
}
