use crate::{prelude::*, Heading};

impl Heading {
    pub fn to_jats_special(&self) -> (String, Losses) {
        use codec_jats_trait::encode::elem;

        let (content, mut losses) = self.content.to_jats();

        // The `level` attribute is not part of the JATS standard but allows
        // lossless roundtrip conversion of Stencila documents to/from JATS.
        let attrs = [("level", &self.level)];

        if self.id.is_some() {
            losses.add("Heading.id")
        }

        (elem("title", attrs, content), losses)
    }

    pub fn to_html_special(&self, context: &mut HtmlEncodeContext) -> String {
        use codec_html_trait::encode::{attr, elem};
        elem(
            &["h", &self.level.max(1).min(6).to_string()].concat(),
            &[attr("id", &self.id.to_html_attr(context))],
            &[self.content.to_html(context)],
        )
    }

    pub fn to_markdown_special(&self, context: &mut MarkdownEncodeContext) -> (String, Losses) {
        let mut md = "#".repeat(self.level.max(1).min(6) as usize);
        md.push(' ');

        let (content, mut losses) = self.content.to_markdown(context);
        md.push_str(&content);

        md.push_str("\n\n");

        if self.id.is_some() {
            losses.add("Heading.id")
        }

        (md, losses)
    }
}
