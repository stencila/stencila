use crate::{Page, prelude::*};

impl DomCodec for Page {
    fn to_dom(&self, context: &mut DomEncodeContext) {
        context.enter_node(self.node_type(), self.node_id());

        self.code.to_dom_attr("code", context);

        context.push_attr("class", &self.code);
        self.content.to_dom(context);

        context.exit_node();
    }
}

impl MarkdownCodec for Page {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context.enter_node(self.node_type(), self.node_id());

        if context.render || !matches!(context.format, Format::Smd) {
            context
                .push_prop_fn(NodeProperty::Content, |context| {
                    self.content.to_markdown(context)
                })
                .exit_node();

            return;
        }

        context
            .push_colons()
            .push_str(" page ")
            .push_prop_str(NodeProperty::Code, &self.code)
            .push_str("\n\n")
            .increase_depth()
            .push_prop_fn(NodeProperty::Content, |context| {
                self.content.to_markdown(context)
            })
            .decrease_depth()
            .push_colons()
            .newline()
            .exit_node()
            .newline();
    }
}
