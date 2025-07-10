use codec_info::lost_options;

use crate::{prelude::*, QuoteBlock};

impl LatexCodec for QuoteBlock {
    fn to_latex(&self, context: &mut LatexEncodeContext) {
        const ENVIRON: &str = "quote";

        context
            .ensure_blankline()
            .enter_node(self.node_type(), self.node_id())
            .environ_begin(ENVIRON)
            .property_fn(NodeProperty::Content, |context| {
                self.content.to_latex(context)
            })
            .environ_end(ENVIRON)
            .newline()
            .exit_node()
            .newline();
    }
}

impl MarkdownCodec for QuoteBlock {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id, source));

        if self.content.is_empty() {
            context.push_str(">");
        } else {
            context
                .push_line_prefix("> ")
                .prefix_empty_lines(true)
                .push_prop_fn(NodeProperty::Content, |context| {
                    self.content.to_markdown(context)
                })
                .trim_end_matches(|char| char == '\n' || char == ' ' || char == '>')
                .prefix_empty_lines(false)
                .pop_line_prefix();
        }

        context.newline().exit_node().newline();
    }
}
