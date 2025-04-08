use crate::{prelude::*, Excerpt};

impl MarkdownCodec for Excerpt {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context.enter_node(self.node_type(), self.node_id());

        if matches!(context.format, Format::Smd | Format::Llmd) {
            context.push_colons().push_str(" excerpt");

            if matches!(context.format, Format::Llmd) {
                context.push_str(" ").push_str(&self.node_id().to_string());
            }

            context.push_str("\n\n").increase_depth();
        }

        context.push_prop_fn(NodeProperty::Content, |context| {
            self.content.to_markdown(context)
        });

        if matches!(context.format, Format::Smd | Format::Llmd) {
            context
                .decrease_depth()
                .push_colons()
                .newline()
                .exit_node()
                .newline();
        } else {
            context.exit_node();
        }
    }
}
