use crate::{
    prelude::*, Article, Block, Primitive, PropertyValue, PropertyValueOrString, Section,
    SectionType,
};

impl Article {
    /// Get the DOI of an article (if any)
    pub fn doi(&self) -> Option<String> {
        const URL_PREFIX: &str = "https://doi.org/";

        let doi = self
            .options
            .identifiers
            .iter()
            .flatten()
            .find_map(|id| match id {
                PropertyValueOrString::PropertyValue(PropertyValue {
                    property_id: Some(property_id),
                    value: Primitive::String(value),
                    ..
                }) => [
                    "doi",
                    URL_PREFIX,
                    "https://registry.identifiers.org/registry/doi",
                ]
                .contains(&property_id.to_lowercase().as_str())
                .then(|| value.to_string().clone()),

                PropertyValueOrString::PropertyValue(PropertyValue {
                    value: Primitive::String(value),
                    ..
                }) => (value.starts_with(URL_PREFIX)).then(|| value.clone()),

                PropertyValueOrString::String(id) => {
                    (id.starts_with(URL_PREFIX)).then(|| id.clone())
                }

                _ => None,
            });

        if let Some(doi) = &doi {
            if !doi.starts_with(URL_PREFIX) {
                return Some([URL_PREFIX, doi].concat());
            }
        }

        doi
    }

    pub fn to_jats_special(&self) -> (String, Losses) {
        use codec_jats_trait::encode::{elem, elem_no_attrs};

        let mut losses = Losses::none();

        // Extract sections from content that belong in <front> or <back>
        // and not in <body>
        let mut front = String::new();
        let mut body = String::new();
        let back = String::new();
        for block in &self.content {
            if let Block::Section(Section {
                section_type: Some(SectionType::Abstract),
                content,
                ..
            }) = block
            {
                let (abstract_jats, abstract_losses) = content.to_jats();
                front.push_str(&elem_no_attrs("abstract", abstract_jats));
                losses.merge(abstract_losses);
            } else {
                let (block_jats, block_losses) = block.to_jats();
                body.push_str(&block_jats);
                losses.merge(block_losses);
            }
        }

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

        self.content.to_latex(context);

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
