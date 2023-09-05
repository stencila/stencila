use codec_html_trait::encode::{attr, elem};

use crate::{prelude::*, Heading};

impl Heading {
    pub fn to_html_special(&self) -> String {
        elem(
            &["h", &self.depth.max(1).min(6).to_string()].concat(),
            &[attr("id", &self.id.to_html_attr())],
            &[self.content.to_html()],
        )
    }

    pub fn to_markdown_special(&self) -> (String, Losses) {
        let mut md = "#".repeat(self.depth.max(1).min(6) as usize);
        md.push(' ');

        let (content, mut losses) = self.content.to_markdown();
        md.push_str(&content);

        md.push_str("\n\n");

        if self.id.is_some() {
            losses.add(Loss::of_properties(
                LossDirection::Encode,
                "Heading",
                ["id".to_string()],
            ))
        }

        (md, losses)
    }
}
