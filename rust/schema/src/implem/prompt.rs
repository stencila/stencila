use common::serde_yaml;
use node_strip::{StripNode, StripTargets};

use crate::{prelude::*, Prompt};

impl MarkdownCodec for Prompt {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        // Based on `Article::to_markdown` but with some differences
        // (e.g. not supporting authors)

        context.enter_node(self.node_type(), self.node_id());

        // Create a header version of self that has no content and can be stripped
        let mut header = Self {
            // Avoid serializing content unnecessarily
            content: Vec::new(),
            ..self.clone()
        };

        // Strip properties from header that are designated as not supported by Markdown.
        // This would be better to do based on the "patch formats" declaration in the
        // schema but that is not accessible from here. So we have to do it "manually"
        header.strip(&StripTargets {
            scopes: vec![
                StripScope::Provenance,
                StripScope::Execution,
                StripScope::Code,
                StripScope::Output,
                StripScope::Archive,
            ],
            ..Default::default()
        });
        header.options.authors = None;

        let mut yaml = serde_yaml::to_value(header).unwrap_or_default();
        if let Some(yaml) = yaml.as_mapping_mut() {
            // Remove the (now empty) content array
            yaml.remove("content");

            // Encode YAML header
            let yaml = serde_yaml::to_string(&yaml).unwrap_or_default();
            context.push_str("---\n");
            context.push_str(&yaml);
            context.push_str("---\n\n");
        }

        context.push_prop_fn(NodeProperty::Content, |context| {
            self.content.to_markdown(context)
        });

        context.append_footnotes();

        context.exit_node_final();
    }
}
