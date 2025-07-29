use codec_markdown_trait::to_markdown;
use common::serde_yaml;

use crate::{
    prelude::*, replicate, shortcuts::t, Article, Block, Collection, CollectionOptions,
    CreativeWorkType, Inline, RawBlock, Reference, SoftwareSourceCode,
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

    /// Get the `is_part_of` property of an article, or generate it from its
    /// `repository` property, if any
    pub fn is_part_of(&self) -> Option<Box<CreativeWorkType>> {
        if let Some(is_part_of) = &self.options.is_part_of {
            return replicate(is_part_of).ok().map(Box::new);
        };

        if let Some(repo) = &self.options.repository {
            if let Some(name) = repo
                .strip_prefix("https://github.com/")
                .or_else(|| repo.strip_prefix("https://gitlab.com/"))
            {
                return Some(Box::new(CreativeWorkType::SoftwareSourceCode(
                    SoftwareSourceCode {
                        name: name.to_string(),
                        repository: Some(repo.clone()),
                        ..Default::default()
                    },
                )));
            } else {
                return Some(Box::new(CreativeWorkType::Collection(Collection {
                    options: Box::new(CollectionOptions {
                        url: Some(repo.clone()),
                        ..Default::default()
                    }),
                    ..Default::default()
                })));
            }
        }

        None
    }

    pub fn to_jats_special(&self) -> (String, Losses) {
        use codec_jats_trait::encode::{elem, elem_no_attrs};

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

        self.date_published.to_dom_attr("date-published", context);
        self.date_accepted.to_dom_attr("date-accepted", context);
        self.date_created.to_dom_attr("date-created", context);

        self.options.is_part_of.to_dom_attr("is-part-of", context);
        self.options.page_start.to_dom_attr("page-start", context);
        self.options.page_end.to_dom_attr("page_end", context);

        self.options.repository.to_dom_attr("repository", context);
        self.options.path.to_dom_attr("path", context);
        self.options.commit.to_dom_attr("commit", context);

        if context.is_root() {
            if let Some(title) = &self.title {
                context.push_slot_fn("section", "title", |context| {
                    context.enter_elem("h1");
                    title.to_dom(context);
                    context.exit_elem();
                });
            }

            if let Some(authors) = &self.authors {
                context.push_slot_fn("div", "authors", |context| authors.to_dom(context));
            }
        } else {
            // If this article is not the root (e.g an article output from an
            // OpenAlex DocsQL query) then represent as a reference
            let reference = Reference::from(self);
            context.push_slot_fn("div", "reference", |context| reference.to_dom(context));
        }

        if let Some(r#abstract) = &self.r#abstract {
            context.push_slot_fn("section", "abstract", |context| r#abstract.to_dom(context));
        }

        if let Some(headings) = &self.headings {
            context.push_slot_fn("nav", "headings", |context| headings.to_dom(context));
        }

        if context.is_root() {
            // Do not show content for articles that are not the root (they
            // usually don't have any content anyway)
            context.push_slot_fn("section", "content", |context| self.content.to_dom(context));
        }

        if let Some(references) = &self.references {
            context.push_slot_fn("section", "references", |context| {
                references.to_dom(context)
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
                .or(self.date_modified.as_ref())
                .or(self.date_created.as_ref())
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

            if let Some(keywords) = &self.keywords {
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

        let yaml = if let Some(yaml) = &self.frontmatter {
            // Front matter is already defined for the article so just use that
            yaml.clone()
        } else if self.title.is_some() || self.r#abstract.is_some() {
            // If there are frontmatter related properties on the article, create YAML frontmatter
            // See `rust/codec-markdown/src/decode/frontmatter.rs` for how frontmatter is decoded.
            // This should be compatible with that if possible
            let mut yaml = serde_yaml::Mapping::new();

            if let Some(title) = &self.title {
                yaml.insert("title".into(), to_markdown(title).into());
            }

            if let Some(date_published) = &self.date_published {
                yaml.insert("date".into(), date_published.value.clone().into());
            }

            if let Some(r#abstract) = &self.r#abstract {
                yaml.insert("abstract".into(), to_markdown(r#abstract).into());
            }

            serde_yaml::to_string(&yaml)
                .unwrap_or_default()
                .trim()
                .to_string()
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

        context.append_footnotes();

        context.exit_node_final();
    }
}
