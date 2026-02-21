use crate::{Workflow, prelude::*};

impl MarkdownCodec for Workflow {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context.enter_node(self.node_type(), self.node_id());

        if let Some(raw_yaml) = &self.frontmatter
            && !raw_yaml.is_empty()
        {
            context.push_prop_fn(NodeProperty::Frontmatter, |context| {
                context.push_str("---\n");
                context.push_str(raw_yaml);
                context.push_str("\n---\n\n");
            });
        }

        if let Some(content) = &self.content {
            context.push_prop_fn(NodeProperty::Content, |context| {
                content.to_markdown(context)
            });
        }

        context.append_footnotes();

        context.exit_node_final();
    }
}
