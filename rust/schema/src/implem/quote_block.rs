use stencila_codec_info::lost_options;

use crate::{QuoteBlock, prelude::*};

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
            // Add indentation for SMD format
            if matches!(context.format, Format::Smd) {
                context.push_indent();
            }
            context.push_str(">").newline();
        } else {
            // For SMD format, include indentation in the line prefix
            let prefix = if matches!(context.format, Format::Smd) {
                [&" ".repeat(context.depth * 4), "> "].concat()
            } else {
                "> ".to_string()
            };

            context
                .push_line_prefix(&prefix)
                .prefix_empty_lines(true)
                .push_prop_fn(NodeProperty::Content, |context| {
                    self.content.to_markdown(context)
                });

            // Clean up trailing prefix if the content ends with an empty prefixed line
            // Only remove the specific prefix we added, not parent prefixes
            let trailing_pattern = format!("{}\n", prefix);
            if context.content.ends_with(&trailing_pattern) {
                for _ in 0..trailing_pattern.len() {
                    context.content.pop();
                }
                // Don't add newline back here - the subsequent newline() calls will handle spacing
            }

            context.prefix_empty_lines(false).pop_line_prefix();
        }

        context.exit_node().newline();
    }
}
