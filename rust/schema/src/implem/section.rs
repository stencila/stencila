use codec_info::lost_options;

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
}

impl MarkdownCodec for Section {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id));

        if let Some(section) = &self.section_type {
            context
                .push_semis()
                .push_str(" ")
                .push_prop_str("section_type", &section.to_string().to_lowercase());
        } else {
            context.push_semis().push_str(" section");
        }

        context
            .push_str("\n\n")
            .increase_depth()
            .push_prop_fn("content", |context| self.content.to_markdown(context))
            .decrease_depth()
            .push_semis()
            .newline()
            .exit_node()
            .newline();
    }
}
