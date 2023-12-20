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

        context.push_prop_fn("content", |context| self.content.to_markdown(context));

        context.exit_node();
        /*
        use common::serde_yaml;

        let mut md = String::new();

        let mut yaml = serde_yaml::to_value(Self {
            // Avoid serializing content
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
                md += "---\n";
                md += &yaml;
                md += "---\n\n";
            }
        }

        let (content_md, losses) = self.content.to_markdown(context);

        md += &content_md;

        md += &context
            .footnotes
            .iter()
            .enumerate()
            .map(|(footnote_index, footnote)| {
                footnote
                    .trim()
                    .lines()
                    .enumerate()
                    .map(|(line_index, line)| {
                        if line_index == 0 {
                            // Place footnote label at start of first line
                            format!("[^{index}]: {line}", index = footnote_index + 1)
                        } else if !line.trim().is_empty() {
                            // Indent subsequent lines by *four* spaces if it is not blank
                            format!("    {line}")
                        } else {
                            // Blank line
                            String::new()
                        }
                    })
                    .join("\n")
            })
            .join("\n\n");

        (md, losses)
        */
    }
}
