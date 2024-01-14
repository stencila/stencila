use common::serde_yaml;

use crate::{prelude::*, Article};

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

        let mut yaml = serde_yaml::to_value(Self {
            //Avoid serializing content unnecessarily
            content: Vec::new(),
            ..self.clone()
        })
        .unwrap_or_default();

        if let Some(yaml) = yaml.as_mapping_mut() {
            // Remove the type and (empty array) content
            yaml.remove("type");
            yaml.remove("content");

            // Only add a YAML header if there are remaining keys
            if !yaml.is_empty() {
                let yaml = serde_yaml::to_string(&yaml).unwrap_or_default();
                context.push_str("---\n");
                context.push_str(&yaml);
                context.push_str("---\n\n");
            }
        }

        context.push_prop_fn("content", |context| self.content.to_markdown(context));

        context.append_footnotes();

        context.exit_node();
    }
}
