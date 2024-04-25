use codec_info::lost_options;

use crate::{prelude::*, QuoteBlock};

impl MarkdownCodec for QuoteBlock {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id, cite))
            .push_line_prefix("> ")
            .prefix_empty_lines(true)
            .push_prop_fn("content", |context| self.content.to_markdown(context))
            .trim_end_matches(|char| char == '\n' || char == ' ' || char == '>')
            .prefix_empty_lines(false)
            .pop_line_prefix()
            .newline()
            .exit_node()
            .newline();
    }
}
