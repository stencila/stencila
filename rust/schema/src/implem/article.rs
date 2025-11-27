use std::collections::BTreeMap;

use stencila_codec_markdown_trait::to_markdown_with;
use stencila_codec_text_trait::to_text;

use crate::{
    Article, Block, CreativeWorkType, Heading, Inline, RawBlock, Reference, Text,
    prelude::*,
    replicate,
    shortcuts::{h1, t},
};

impl Article {
    /// Does the article appear to be have been decoded from the format using the `--coarse` option
    ///
    /// Checks whether the first block in the content of the article is a `RawBlock` of the given formats
    pub fn is_coarse(&self, format: &Format) -> bool {
        if let Some(Block::RawBlock(raw)) = self.content.first() {
            &Format::from_name(&raw.format) == format
        } else {
            false
        }
    }

    /// Get the `title` property of an article, or generate it from its
    /// `path` property, if any
    pub fn title(&self) -> Option<Vec<Inline>> {
        if let Some(title) = &self.title {
            return replicate(title).ok();
        };

        if let Some(path) = &self.options.path {
            return Some(vec![t(path.to_string())]);
        }

        None
    }

    /// Create a [`Reference`] from the `is_part_of` of an article, or from its
    /// `repository` property, if any
    pub fn is_part_of(&self) -> Option<Reference> {
        if let Some(is_part_of) = &self.options.is_part_of {
            Some(Reference::from(is_part_of))
        } else if let Some(repo) = self.options.repository.clone() {
            if let Some(name) = repo
                .strip_prefix("https://github.com/")
                .or_else(|| repo.strip_prefix("https://gitlab.com/"))
            {
                Some(Reference {
                    work_type: Some(CreativeWorkType::SoftwareRepository),
                    title: Some(vec![t(name)]),
                    url: Some(repo),
                    ..Default::default()
                })
            } else {
                Some(Reference {
                    url: Some(repo),
                    ..Default::default()
                })
            }
        } else {
            None
        }
    }

    /// Generate document-level CSS variables from article metadata
    ///
    /// Extracts metadata like title, authors, dates, and DOI into CSS variable
    /// name/value pairs (without the `--` prefix). These can be used for print
    /// headers/footers or injected into computed theme variables.
    pub fn document_variables(&self) -> BTreeMap<String, String> {
        let mut vars = BTreeMap::new();

        if let Some(title) = &self.title {
            let mut title = to_text(title).replace("\"", "'");
            const MAX_LEN: usize = 120;
            if title.len() > MAX_LEN {
                title.truncate(MAX_LEN);
                title.push('…');
            }
            vars.insert("document-title".to_string(), title);
        }

        if let Some(authors) = &self.authors {
            let authors = match authors.len() {
                0 => String::new(),
                1 => authors[0].short_name(),
                2 => [&authors[0].short_name(), " & ", &authors[1].short_name()].concat(),
                _ => [&authors[0].short_name(), " et al."].concat(),
            };
            vars.insert("document-authors".to_string(), authors.replace("\"", "'"));
        }

        if let Some(date) = self
            .date_published
            .as_ref()
            .or(self.options.date_modified.as_ref())
            .or(self.options.date_accepted.as_ref())
            .or(self.options.date_received.as_ref())
            .or(self.options.date_created.as_ref())
        {
            vars.insert("document-date".to_string(), date.value.replace("\"", "'"));
        }

        if let Some(doi) = &self.doi {
            vars.insert(
                "document-doi".to_string(),
                format!("DOI: {}", doi.replace("\"", "'")),
            );
        }

        vars
    }

    pub fn to_jats_special(&self) -> (String, Losses) {
        use stencila_codec_jats_trait::encode::{elem, elem_no_attrs};

        let mut losses = Losses::none();

        let mut front = String::new();
        if let Some(content) = &self.r#abstract {
            let (abstract_jats, abstract_losses) = content.to_jats();
            front.push_str(&elem_no_attrs("abstract", abstract_jats));
            losses.merge(abstract_losses);
        }

        let mut body = String::new();
        for block in &self.content {
            let (block_jats, block_losses) = block.to_jats();
            body.push_str(&block_jats);
            losses.merge(block_losses);
        }

        let back = String::new();

        let mut content = String::new();
        if !front.is_empty() {
            content.push_str(&elem_no_attrs("front", front));
        }
        if !body.is_empty() {
            content.push_str(&elem_no_attrs("body", body));
        }
        if !back.is_empty() {
            content.push_str(&elem_no_attrs("back", back));
        }

        (
            elem(
                "article",
                [
                    ("dtd-version", "1.3"),
                    ("xmlns:xlink", "http://www.w3.org/1999/xlink"),
                    ("xmlns:mml", "http://www.w3.org/1998/Math/MathML"),
                ],
                content,
            ),
            losses,
        )
    }
}

impl DomCodec for Article {
    fn to_dom(&self, context: &mut DomEncodeContext) {
        context.enter_node(self.node_type(), self.node_id());

        self.doi.to_dom_attr("doi", context);
        self.options.identifiers.to_dom_attr("identifiers", context);

        self.options
            .date_created
            .to_dom_attr("date-created", context);
        self.options
            .date_modified
            .to_dom_attr("date-modified", context);
        self.options
            .date_received
            .to_dom_attr("date-received", context);
        self.options
            .date_accepted
            .to_dom_attr("date-accepted", context);
        self.date_published.to_dom_attr("date-published", context);

        self.options.is_part_of.to_dom_attr("is-part-of", context);
        self.options.page_start.to_dom_attr("page-start", context);
        self.options.page_end.to_dom_attr("page-end", context);

        self.options.repository.to_dom_attr("repository", context);
        self.options.path.to_dom_attr("path", context);
        self.options.commit.to_dom_attr("commit", context);

        if context.is_root() {
            // Generate CSS variables for print media support from document metadata
            let doc_vars = self.document_variables();
            if !doc_vars.is_empty() {
                let mut css = String::new();
                for (name, value) in doc_vars {
                    css.push_str(&format!("--{name}: \"{value}\";"));
                }
                context.push_css(&[":root {", &css, "}"].concat());
            }

            if let Some(title) = &self.title {
                // We do not use <h1> or <header><p role="heading" aria-level="1"> for title
                // because  bot result in the title being treated as a level one header
                // when generating a PDF. Instead we use the ARIA "banner" role.
                context.push_slot_fn("header", "title", |context| {
                    context.push_attr("role", "banner").enter_elem("p");
                    title.to_dom(context);
                    context.exit_elem();
                });
            }

            if let Some(authors) = &self.authors {
                context.push_slot_fn("section", "authors", |context| {
                    for (index, author) in authors.iter().enumerate() {
                        if index > 0 {
                            context.push_html(", ");
                        }
                        context
                            .enter_node(author.node_type(), author.node_id())
                            .push_slot_fn("span", "name", |context| {
                                context.push_text(&author.name());
                            })
                            .exit_node();
                    }
                });
            }
        } else {
            // If this article is not the root (e.g an article output from an
            // OpenAlex DocsQL query) then represent as a reference
            let reference = Reference::from(self);
            context.push_slot_fn("div", "reference", |context| reference.to_dom(context));
        }

        if let Some(r#abstract) = &self.r#abstract {
            // For consistency with sections in the `content` render as a
            // <stencila-section> with a heading if necessary
            context.push_slot_fn("section", "abstract", |context| {
                context
                    .enter_node(NodeType::Section, NodeId::new(b"sec", b"abstract"))
                    .push_slot_fn("section", "content", |context| {
                        // Add an abstract heading if one does not yet exist
                        if !r#abstract.iter().any(|block| match block {
                            Block::Heading(Heading { content, .. }) => {
                                content.iter().any(|inline| match inline {
                                    Inline::Text(Text { value, .. }) => {
                                        value.to_lowercase() == "abstract"
                                    }
                                    _ => false,
                                })
                            }
                            _ => false,
                        }) {
                            h1([t("Abstract")]).to_dom(context);
                        }

                        r#abstract.to_dom(context)
                    })
                    .exit_node();
            });
        }

        if context.is_root()
            && let Some(headings) = &self.options.headings
        {
            context.push_slot_fn("nav", "headings", |context| headings.to_dom(context));
        }

        if !self.content.is_empty() {
            context.push_slot_fn("section", "content", |context| self.content.to_dom(context));
        }

        if let Some(references) = &self.references
            && !references.is_empty()
        {
            // For consistency with sections in the `content` render as a
            // <stencila-section> with a heading
            context.push_slot_fn("section", "references", |context| {
                context
                    .enter_node(NodeType::Section, NodeId::new(b"sec", b"references"))
                    .push_slot_fn("section", "content", |context| {
                        h1([t("References")]).to_dom(context);
                        references.to_dom(context)
                    })
                    .exit_node();
            });
        }

        context.exit_node();
    }
}

impl LatexCodec for Article {
    fn to_latex(&self, context: &mut LatexEncodeContext) {
        context.enter_node(self.node_type(), self.node_id());

        // Scan any raw latex blocks to check if command is already present
        let has = |what: &str| -> bool {
            self.content.iter().any(|block| match block {
                Block::RawBlock(RawBlock {
                    format, content, ..
                }) => matches!(Format::from_name(format), Format::Latex) && content.contains(what),
                _ => false,
            })
        };

        const ENVIRON: &str = "document";
        if context.standalone {
            if !has("\\documentclass") {
                context.str("\\documentclass{article}\n\n");
            }

            if let Some(title) = &self.title {
                context.property_fn(NodeProperty::Title, |context| {
                    context.command_begin("title");
                    title.to_latex(context);
                    context.command_end().newline();
                });
                context.newline();
            }

            if let Some(authors) = &self.authors {
                context.property_fn(NodeProperty::Authors, |context| {
                    for author in authors {
                        context
                            .command_begin("author")
                            .escaped_str(&author.name())
                            .command_end()
                            .newline();
                    }
                });
                context.newline();
            }

            if let Some(date) = self
                .date_published
                .as_ref()
                .or(self.options.date_modified.as_ref())
                .or(self.options.date_accepted.as_ref())
                .or(self.options.date_received.as_ref())
                .or(self.options.date_created.as_ref())
            {
                context.property_fn(NodeProperty::Date, |context| {
                    context
                        .command_begin("date")
                        .escaped_str(&date.value)
                        .command_end()
                        .newline();
                });
                context.newline();
            }

            if let Some(keywords) = &self.options.keywords {
                context.property_fn(NodeProperty::Keywords, |context| {
                    context
                        .command_begin("keywords")
                        .escaped_str(&keywords.join(", "))
                        .command_end()
                        .newline();
                });
                context.newline();
            }

            if !has("\\begin{document}") {
                context.environ_begin(ENVIRON).str("\n\n");
            }

            if self.title.is_some() {
                context.str("\\maketitle\n\n");
            }
        }

        context.property_fn(NodeProperty::Content, |context| {
            self.content.to_latex(context)
        });

        if context.standalone && !has("\\end{document}") {
            context.ensure_blankline().environ_end(ENVIRON).char('\n');
        }

        context.exit_node_final();
    }
}

impl MarkdownCodec for Article {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context.enter_node(self.node_type(), self.node_id());

        let yaml = if self.title.is_some() || self.r#abstract.is_some() {
            // If there are frontmatter related properties on the article, create, or update existing, YAML frontmatter
            // See `rust/codec-markdown/src/decode/frontmatter.rs` for how frontmatter is decoded.
            // This should be compatible with that if possible
            let mut yaml: Option<serde_yaml::Mapping> = if let Some(yaml) = &self.frontmatter {
                // Parse existing frontmatter so it can be updated
                serde_yaml::from_str(yaml).ok()
            } else {
                // Start with empty frontmatter
                Some(serde_yaml::Mapping::new())
            };

            if let Some(yaml) = &mut yaml {
                // Update the title and abstract of the work, which may include executable code expressions,
                // which if render: true will be encoded differently from in any original template.

                // Track whether title or abstract changed to avoid unnecessary reformatting
                let mut title_changed = false;
                let mut abstract_changed = false;

                if let Some(title) = &self.title {
                    let new_markdown =
                        to_markdown_with(title, context.format.clone(), context.render);

                    // Only update if the content has actually changed
                    let should_update =
                        if let Some(existing) = yaml.get("title").and_then(|v| v.as_str()) {
                            existing.trim() != new_markdown.trim()
                        } else {
                            true // No existing title, definitely update
                        };

                    if should_update {
                        yaml.insert("title".into(), new_markdown.into());
                        title_changed = true;
                    }
                }

                if let Some(date_published) = &self.date_published {
                    yaml.insert("date".into(), date_published.value.clone().into());
                }

                if let Some(r#abstract) = &self.r#abstract {
                    let new_markdown =
                        to_markdown_with(r#abstract, context.format.clone(), context.render);

                    // Only update if the content has actually changed
                    let should_update =
                        if let Some(existing) = yaml.get("abstract").and_then(|v| v.as_str()) {
                            existing.trim() != new_markdown.trim()
                        } else {
                            true // No existing abstract, definitely update
                        };

                    if should_update {
                        yaml.insert("abstract".into(), new_markdown.into());
                        abstract_changed = true;
                    }
                }

                // If neither title nor abstract changed, return original frontmatter to avoid reformatting
                if !title_changed && !abstract_changed && self.frontmatter.is_some() {
                    self.frontmatter.clone().unwrap_or_default()
                } else {
                    serde_yaml::to_string(&yaml)
                        .unwrap_or_default()
                        .trim()
                        .to_string()
                }
            } else {
                // Should only end up here if there is already frontmatter but
                // that errored when parsed. So just return it  verbatim,
                // without trying to update it.
                self.frontmatter.clone().unwrap_or_default()
            }
        } else if let Some(yaml) = &self.frontmatter {
            // Front matter is already defined for the article so just use that
            yaml.clone()
        } else {
            String::new()
        };

        if !yaml.is_empty() {
            context.push_prop_fn(NodeProperty::Frontmatter, |context| {
                context.push_str("---\n");
                context.push_str(&yaml);
                context.push_str("\n---\n\n");
            });
        }

        context.push_prop_fn(NodeProperty::Content, |context| {
            self.content.to_markdown(context)
        });

        if context.render
            && let Some(references) = &self.references
            && !references.is_empty()
        {
            context.push_prop_fn(NodeProperty::References, |context| {
                context.push_str("# References\n\n");
                references.to_markdown(context);
            });
        }

        context.append_footnotes();

        context.exit_node_final();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Author, Date, Person};

    #[test]
    fn test_document_variables_empty() {
        let article = Article::default();
        let vars = article.document_variables();
        assert!(vars.is_empty());
    }

    #[test]
    fn test_document_variables_title() {
        let article = Article {
            title: Some(vec![t("Test Article Title")]),
            ..Default::default()
        };
        let vars = article.document_variables();
        assert_eq!(
            vars.get("document-title"),
            Some(&"Test Article Title".to_string())
        );
    }

    #[test]
    fn test_document_variables_title_truncation() {
        let long_title = "a".repeat(150);
        let article = Article {
            title: Some(vec![t(&long_title)]),
            ..Default::default()
        };
        let vars = article.document_variables();
        let title = vars.get("document-title").expect("should have title");
        // Ellipsis is 3 bytes in UTF-8, so 120 chars + 3 byte ellipsis = 123
        assert_eq!(title.len(), 123);
        assert!(title.ends_with('…'));
    }

    #[test]
    fn test_document_variables_single_author() {
        let article = Article {
            authors: Some(vec![Author::Person(Person {
                given_names: Some(vec!["Jane".to_string()]),
                family_names: Some(vec!["Doe".to_string()]),
                ..Default::default()
            })]),
            ..Default::default()
        };
        let vars = article.document_variables();
        assert_eq!(vars.get("document-authors"), Some(&"Doe".to_string()));
    }

    #[test]
    fn test_document_variables_two_authors() {
        let article = Article {
            authors: Some(vec![
                Author::Person(Person {
                    given_names: Some(vec!["Jane".to_string()]),
                    family_names: Some(vec!["Doe".to_string()]),
                    ..Default::default()
                }),
                Author::Person(Person {
                    given_names: Some(vec!["John".to_string()]),
                    family_names: Some(vec!["Smith".to_string()]),
                    ..Default::default()
                }),
            ]),
            ..Default::default()
        };
        let vars = article.document_variables();
        assert_eq!(
            vars.get("document-authors"),
            Some(&"Doe & Smith".to_string())
        );
    }

    #[test]
    fn test_document_variables_three_authors() {
        let article = Article {
            authors: Some(vec![
                Author::Person(Person {
                    given_names: Some(vec!["Jane".to_string()]),
                    family_names: Some(vec!["Doe".to_string()]),
                    ..Default::default()
                }),
                Author::Person(Person {
                    given_names: Some(vec!["John".to_string()]),
                    family_names: Some(vec!["Smith".to_string()]),
                    ..Default::default()
                }),
                Author::Person(Person {
                    given_names: Some(vec!["Bob".to_string()]),
                    family_names: Some(vec!["Jones".to_string()]),
                    ..Default::default()
                }),
            ]),
            ..Default::default()
        };
        let vars = article.document_variables();
        assert_eq!(
            vars.get("document-authors"),
            Some(&"Doe et al.".to_string())
        );
    }

    #[test]
    fn test_document_variables_date() {
        let article = Article {
            date_published: Some(Date {
                value: "2025-01-15".to_string(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let vars = article.document_variables();
        assert_eq!(vars.get("document-date"), Some(&"2025-01-15".to_string()));
    }

    #[test]
    fn test_document_variables_doi() {
        let article = Article {
            doi: Some("10.1234/test.2025".to_string()),
            ..Default::default()
        };
        let vars = article.document_variables();
        assert_eq!(
            vars.get("document-doi"),
            Some(&"DOI: 10.1234/test.2025".to_string())
        );
    }

    #[test]
    fn test_document_variables_all_fields() {
        let article = Article {
            title: Some(vec![t("Complete Test")]),
            authors: Some(vec![Author::Person(Person {
                given_names: Some(vec!["Jane".to_string()]),
                family_names: Some(vec!["Doe".to_string()]),
                ..Default::default()
            })]),
            date_published: Some(Date {
                value: "2025-01-15".to_string(),
                ..Default::default()
            }),
            doi: Some("10.1234/test".to_string()),
            ..Default::default()
        };
        let vars = article.document_variables();
        assert_eq!(vars.len(), 4);
        assert!(vars.contains_key("document-title"));
        assert!(vars.contains_key("document-authors"));
        assert!(vars.contains_key("document-date"));
        assert!(vars.contains_key("document-doi"));
    }
}
