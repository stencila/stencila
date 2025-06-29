use codec_latex_trait::{latex_to_image, to_latex};
use common::tracing;

use crate::{prelude::*, Island, LabelType};

impl LatexCodec for Island {
    fn to_latex(&self, context: &mut LatexEncodeContext) {
        context.enter_node(self.node_type(), self.node_id());

        if matches!(context.format, Format::Docx | Format::Odt) {
            let (mut latex, ..) = to_latex(
                &self.content,
                Format::Svg,
                false,
                true,
                false,
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
            let inner = if let Err(error) = latex_to_image(&latex, &path, self.style.as_deref()) {
                tracing::error!("While encoding island to image: {error}\n\n{latex}");

                // Rather than adding potentially broken LaTeX to DOCX/ODT, add message to document
                r"\verb|[Unable to generate image from LaTeX. Please refer to PDF or other version]|".to_string()
            } else {
                let path = path.to_string_lossy();
                [r"\includegraphics[width=16cm]{", &path, "}"].concat()
            };

            context.str(r"\centerline{");

            // Add id (if any) any as a label to that cross links work
            if let Some(id) = &self.id {
                context.str(r"\label{").str(id).char('}');
            }

            if context.reproducible {
                context.link_begin(None);
            }

            context.str(&inner);

            if context.reproducible {
                context.link_end();
            }

            context.str("}").exit_node();

            return;
        }

        const ENVIRON: &str = "island";
        let should_wrap = !self.is_automatic.unwrap_or_default()
            && matches!(context.format, Format::Latex | Format::Tex);
        if should_wrap {
            context.environ_begin(ENVIRON);

            let has_options = self.id.is_some()
                || self.label_type.is_some()
                || self.label.is_some()
                || self.style.is_some();
            if has_options {
                let props = [
                    self.id.clone(),
                    self.label_type.as_ref().and_then(|lt| {
                        if let Some(id) = &self.id {
                            // Label type does not need to be encoded if id contains it
                            if id.starts_with("tab:")
                                && matches!(self.label_type, None | Some(LabelType::TableLabel))
                                || id.starts_with("fig:")
                                    && matches!(
                                        self.label_type,
                                        None | Some(LabelType::FigureLabel)
                                    )
                            {
                                return None;
                            }
                        }

                        Some(
                            [
                                "label-type=",
                                match lt {
                                    LabelType::FigureLabel => "fig",
                                    LabelType::TableLabel => "tab",
                                },
                            ]
                            .concat(),
                        )
                    }),
                    self.label.as_ref().map(|label| ["label=", label].concat()),
                    self.style.as_ref().map(|style| ["style=", style].concat()),
                ]
                .into_iter()
                .flatten()
                .join(",");

                context.char('[').str(&props).char(']');
            }

            context.str("\n\n");
        }

        context.property_fn(NodeProperty::Content, |context| {
            self.content.to_latex(context)
        });

        if should_wrap {
            context.environ_end(ENVIRON).newline();
        }

        context.exit_node();
    }
}
