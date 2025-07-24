use crate::{prelude::*, Collection, Reference};

impl MarkdownCodec for Collection {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        // When encoding a collection to Markdown, instead of encoding each
        // creative work in its entirety we encode it as a reference. For
        // encoding all of the collection, including the content of each work,
        // we will probably implement special handling of to_path for
        // collections where it is encoded to a directory.
        context
            .enter_node(self.node_type(), self.node_id())
            .push_prop_fn(NodeProperty::Parts, |context| {
                for part in &self.parts {
                    let reference = Reference::from(part);
                    reference.to_markdown(context)
                }
            })
            .exit_node_final();
    }
}
