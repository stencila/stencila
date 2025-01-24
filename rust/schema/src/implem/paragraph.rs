use crate::{prelude::*, Paragraph};

impl LatexCodec for Paragraph {
    fn to_latex(&self, context: &mut LatexEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .property_fn(NodeProperty::Content, |context| {
                self.content.to_latex(context)
            })
            .newline()
            .exit_node()
            .newline();
    }
}
