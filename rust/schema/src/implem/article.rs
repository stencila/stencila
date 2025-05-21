use crate::{prelude::*, Article, Block, RawBlock};

impl Article {
    /// Does tha article appear to be have been decoded from the format using the `--coarse` option
    ///
    /// Checks whether the first block in the content of the article is a `RawBlock` of the given formats
    pub fn is_coarse(&self, format: &Format) -> bool {
        if let Some(Block::RawBlock(raw)) = self.content.first() {
            &Format::from_name(&raw.format) == format
        } else {
            false
        }
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
                    context.command_enter("title");
                    title.to_latex(context);
                    context.command_exit().newline();
                });
                context.newline();
            }

            if let Some(authors) = &self.authors {
                context.property_fn(NodeProperty::Authors, |context| {
                    for author in authors {
                        context
                            .command_enter("author")
                            .escape_str(&author.name())
                            .command_exit()
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
                        .command_enter("date")
                        .escape_str(&date.value)
                        .command_exit()
                        .newline();
                });
                context.newline();
            }

            if let Some(keywords) = &self.keywords {
                context.property_fn(NodeProperty::Keywords, |context| {
                    context
                        .command_enter("keywords")
                        .escape_str(&keywords.join(", "))
                        .command_exit()
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
            if !context.content.ends_with("\n") {
                context.char('\n');
            }
            if !context.content.ends_with("\n\n") {
                context.char('\n');
            }
            context.environ_end(ENVIRON).char('\n');
        }

        context.exit_node_final();
    }
}

impl MarkdownCodec for Article {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context.enter_node(self.node_type(), self.node_id());

        if let Some(yaml) = &self.frontmatter {
            if !yaml.is_empty() {
                context.push_prop_fn(NodeProperty::Frontmatter, |context| {
                    context.push_str("---\n");
                    context.push_str(yaml);
                    context.push_str("\n---\n\n");
                });
            }
        }

        context.push_prop_fn(NodeProperty::Content, |context| {
            self.content.to_markdown(context)
        });

        context.append_footnotes();

        context.exit_node_final();
    }
}
