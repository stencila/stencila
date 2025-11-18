use crate::{ThematicBreak, prelude::*};

impl MarkdownCodec for ThematicBreak {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context.enter_node(self.node_type(), self.node_id());

        // Add indentation for SMD format
        if matches!(context.format, Format::Smd) {
            context.push_indent();
        }

        // Use three asterisks, rather than three hyphens, to avoid confusion
        // with YAML front matter delimiters
        context.push_str("***").newline().exit_node().newline();
    }
}
