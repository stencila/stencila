use crate::{Paragraph, prelude::*};

impl LatexCodec for Paragraph {
    fn to_latex(&self, context: &mut LatexEncodeContext) {
        context
            .ensure_blankline()
            .enter_node(self.node_type(), self.node_id())
            .paragraph_begin()
            .property_fn(NodeProperty::Content, |context| {
                self.content.to_latex(context)
            })
            .paragraph_end()
            .newline()
            .exit_node()
            .newline();
    }
}

impl MarkdownCodec for Paragraph {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        if matches!(context.format, Format::Smd) {
            context.push_indent();
        }

        context
            .enter_node(self.node_type(), self.node_id())
            .push_prop_fn(NodeProperty::Content, |context| {
                self.content.to_markdown(context)
            })
            .newline()
            .exit_node()
            .newline();
    }
}
