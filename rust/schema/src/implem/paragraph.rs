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
