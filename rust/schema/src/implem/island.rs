use codec_info::lost_options;
use codec_latex_trait::{latex_to_image, to_latex};
use common::tracing;

use crate::{prelude::*, Island, LabelType};

impl LatexCodec for Island {
    fn to_latex(&self, context: &mut LatexEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(
                self,
                id,
                label,
                label_type,
                label_automatically
            ));

        if matches!(context.format, Format::Docx | Format::Odt) {
            let (mut latex, ..) = to_latex(
                &self.content,
                Format::Latex,
                false,
                true,
                false,
                context.prelude.clone(),
            );

            if let (Some(label_type), Some(number)) = (
                self.label_type,
                self.label
                    .as_ref()
                    .and_then(|label| label.parse::<u32>().ok()),
            ) {
                let label_type = match label_type {
                    LabelType::FigureLabel => "figure",
                    LabelType::TableLabel => "table",
                };
                let counter = number.saturating_sub(1);

                latex.insert_str(0, &format!(r"\setcounter{{{label_type}}}{{{counter}}}"));
            }

            let path = context.temp_dir.join(format!("{}.svg", self.node_id()));
            if let Err(error) = latex_to_image(&latex, &path, self.style.as_deref()) {
                tracing::error!("While encoding island to image: {error}");
                // Will fallback to just encoding the content below
            } else {
                let path = path.to_string_lossy();

                // Add id (if any) any as a label to that cross links work
                if let Some(id) = &self.id {
                    context.str(r"\label{").str(id).char('}');
                }

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
