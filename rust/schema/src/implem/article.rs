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

        let wrap = context.standalone
            && !self
                .content
                .first()
                .map(|block| match block {
                    Block::RawBlock(RawBlock { content, .. }) => {
                        content.contains(r"\documentclass")
                    }
                    _ => false,
                })
                .unwrap_or_default();

        if wrap {
            context.str(
                r"\documentclass{article}

\begin{document}

",
            );
        }

        self.content.to_latex(context);

        if wrap {
            context.str(r"\end{document}").newline();
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
