use codec_markdown_trait::to_markdown;
use common::serde_yaml;
use node_strip::{StripNode, StripTargets};

use crate::{prelude::*, Article, Author};

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
                StripScope::Archive,
            ],
            ..Default::default()
        });
        header.headings = None;

        // If there is a title, represent it as Markdown
        let mut title_string: Option<String> = None;
        if let Some(title) = &header.title {
            title_string = Some(to_markdown(title))
        }

        // Unwrap `AuthorRoles`. These can be added when the document is authored
        // in some tools but have too many/unnecessary details for a YAML header.
        // Also remove any un-named authors.
        if let Some(authors) = &mut header.authors {
            let mut author_roles = 0;
            for author in authors.iter_mut() {
                if let Author::AuthorRole(role) = author {
                    author_roles += 1;
                    if let Some(inner) = role.to_author() {
                        *author = inner;
                    }
                }
            }
            authors.retain(|author| match author {
                Author::Person(person) => {
                    let has_given_names = person
                        .given_names
                        .as_ref()
                        .map(|names| !names.is_empty())
                        .unwrap_or_default();

                    let has_family_names = person
                        .family_names
                        .as_ref()
                        .map(|names| !names.is_empty())
                        .unwrap_or_default();

                    let has_name = person
                        .options
                        .name
                        .as_ref()
                        .map(|names| !names.is_empty())
                        .unwrap_or_default();

                    has_given_names || has_family_names || has_name
                }
                _ => true,
            });
            // If authors is now empty, or only consist of author roles then make none
            // TODO: this is done to avoid having Markdown font matter populated automatically
            // when editing in Stencila VSCode extension, but there is probably a better approach!
            if authors.is_empty() || author_roles >= authors.len() {
                header.authors = None;
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
