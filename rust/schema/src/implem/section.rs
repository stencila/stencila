use crate::{prelude::*, Section, SectionType};

impl Section {
    pub fn to_html_special(&self, context: &mut HtmlEncodeContext) -> String {
        use codec_html_trait::encode::{attr, elem};

        let (tag, attrs) = match &self.section_type {
            Some(SectionType::Main) => ("main", vec![]),
            Some(SectionType::Header) => ("header", vec![]),
            Some(SectionType::Footer) => ("footer", vec![]),
            Some(typ) => ("section", vec![attr("id", &typ.to_string().to_lowercase())]),
            None => ("section", vec![]),
        };

        let children = self.content.to_html(context);

        elem(tag, &attrs, &[children])
    }

    pub fn to_markdown_special(&self, context: &mut MarkdownEncodeContext) -> (String, Losses) {
        let fence = ":".repeat(3 + context.depth * 2);

        let typ = match &self.section_type {
            Some(typ) => typ.to_string().to_lowercase(),
            None => String::from("section"),
        };

        context.down();
        let (md, losses) = self.content.to_markdown(context);
        context.up();

        let md = [&fence, " ", &typ, "\n\n", &md, &fence, "\n\n"].concat();

        (md, losses)
    }
}
