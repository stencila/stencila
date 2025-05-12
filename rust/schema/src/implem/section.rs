use codec_info::lost_options;
use codec_latex_trait::{latex_to_image, to_latex};
use common::tracing;

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

impl LatexCodec for Section {
    fn to_latex(&self, context: &mut LatexEncodeContext) {
        context.enter_node(self.node_type(), self.node_id());

        if matches!(context.format, Format::Docx | Format::Odt)
            && matches!(self.section_type, Some(SectionType::Island))
        {
            let (latex, ..) = to_latex(
                &self.content,
                Format::Latex,
                false,
                true,
                false,
                context.prelude.clone(),
            );

            let path = context.temp_dir.join(format!("{}.svg", self.node_id()));
            if let Err(error) = latex_to_image(&latex, &path, Some("island")) {
                tracing::error!("While encoding island section to image: {error}");
                // Will fallback to just encoding the content below
            } else {
                let path = path.to_string_lossy();
                context
                    .str(r"\centerline{\includegraphics{")
                    .str(&path)
                    .str("}}")
                    .exit_node();
                return;
            }
        }

        context
            .property_fn(NodeProperty::Content, |context| {
                self.content.to_latex(context)
            })
            .exit_node();
    }
}

impl MarkdownCodec for Section {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id));

        if let Some(section) = &self.section_type {
            context.push_colons().push_str(" ").push_prop_str(
                NodeProperty::SectionType,
                &section.to_string().to_lowercase(),
            );
        } else {
            context.push_colons().push_str(" section");
        }

        context
            .push_str("\n\n")
            .increase_depth()
            .push_prop_fn(NodeProperty::Content, |context| {
                self.content.to_markdown(context)
            })
            .decrease_depth()
            .push_colons()
            .newline()
            .exit_node()
            .newline();
    }
}
