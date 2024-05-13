use common::serde_yaml;

use node_strip::{StripNode, StripTargets};

use crate::{prelude::*, Article, Author, Inline};

impl Article {
    pub fn to_jats_special(&self) -> (String, Losses) {
        use codec_jats_trait::encode::{elem, elem_no_attrs};

        let mut losses = Losses::none();

        let front = String::new(); // TODO elem_no_attrs("front", "");

        let (content_jats, content_losses) = self.content.to_jats();
        let body = elem_no_attrs("body", content_jats);
        losses.merge(content_losses);

        let back = String::new(); // TODO elem_no_attrs("back", "");

        (
            elem(
                "article",
                [
                    ("dtd-version", "1.3"),
                    ("xmlns:xlink", "http://www.w3.org/1999/xlink"),
                    ("xmlns:mml", "http://www.w3.org/1998/Math/MathML"),
                ],
                [front, body, back].concat(),
            ),
            losses,
        )
    }
}

impl MarkdownCodec for Article {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context.enter_node(self.node_type(), self.node_id());

        // Create a header version of self that has no content and can be stripped
        let mut header = Self {
            //Avoid serializing content unnecessarily
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
            ],
            types: vec![],
            properties: vec![],
        });

        // If the title is a single text node then simplify it to a YAML string
        let mut title_string: Option<String> = None;
        if let Some(title) = &header.title {
            if title.len() == 1 {
                if let Some(Inline::Text(text)) = title.first() {
                    title_string = Some(text.value.to_string())
                }
            }
        }

        // Unwrap `AuthorRoles`. These can be added when the document is authored
        // in some tools but have too many/unnecessary details for a YAML header.
        if let Some(authors) = &mut header.authors {
            for author in authors {
                if let Author::AuthorRole(role) = author {
                    if let Some(inner) = role.to_author() {
                        *author = inner;
                    }
                }
            }
        }

        let mut yaml = serde_yaml::to_value(header).unwrap_or_default();
        if let Some(yaml) = yaml.as_mapping_mut() {
            // Remove the type and (the now empty) content array
            yaml.remove("type");
            yaml.remove("content");

            // Set title string if any
            use serde_yaml::Value;
            if let Some(title) = title_string {
                yaml.insert(Value::from("title"), Value::from(title));
            }

            // Only add a YAML header if there are remaining keys
            if !yaml.is_empty() {
                let yaml = serde_yaml::to_string(&yaml).unwrap_or_default();
                context.push_str("---\n");
                context.push_str(&yaml);
                context.push_str("---\n\n");
            }
        }

        context.push_prop_fn(NodeProperty::Content, |context| {
            self.content.to_markdown(context)
        });

        context.append_footnotes();

        context.exit_node_final();
    }
}
