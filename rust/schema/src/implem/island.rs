use stencila_codec_latex_trait::{latex_to_image, to_latex};

use crate::{Island, LabelType, prelude::*};

impl LatexCodec for Island {
    fn to_latex(&self, context: &mut LatexEncodeContext) {
        context.enter_node(self.node_type(), self.node_id());

        if context.has_format_via_pandoc() {
            let (mut latex, ..) = to_latex(&self.content, Format::Svg, false, true, false, false);

            // Set figure or table counter within the island's LaTeX
            if let (Some(label_type), Some(label)) = (self.label_type, &self.label)
                && let Some(label_type) = match label_type {
                    LabelType::FigureLabel => Some("figure"),
                    LabelType::TableLabel => Some("table"),
                    // This should not usually happen
                    _ => None,
                }
            {
                // For islands within appendices (label starts with uppercase number) we need
                // to derive a counter for the appendix as well as for the figure or table
                let (appendix_counter, label_type_counter): (Option<u32>, Option<u32>) = {
                    // Split label into uppercase prefix and numeric suffix
                    let prefix: String = label
                        .chars()
                        .take_while(|c| c.is_ascii_uppercase())
                        .collect();
                    let suffix: String = label
                        .chars()
                        .skip_while(|c| c.is_ascii_uppercase())
                        .collect();

                    // Parse appendix counter from first uppercase letter (A=1, B=2, etc.)
                    let appendix = prefix.chars().next().map(|c| c as u32 - 'A' as u32 + 1);

                    // Parse label counter and subtract 1 (because the figure itself will increment the counter)
                    let label_counter = suffix
                        .parse::<u32>()
                        .ok()
                        .and_then(|n| if n > 0 { Some(n - 1) } else { None });

                    (appendix, label_counter)
                };

                let mut counters = String::new();

                if let Some(appendix_counter) = appendix_counter {
                    counters.push_str("\\appendix\n");
                    counters.push_str("\\setcounter{section}{");
                    counters.push_str(&appendix_counter.to_string());
                    counters.push_str("}\n");
                }

                if let Some(label_type_counter) = label_type_counter {
                    counters.push_str("\\setcounter{");
                    counters.push_str(label_type);
                    counters.push_str("}{");
                    counters.push_str(&label_type_counter.to_string());
                    counters.push_str("}\n");
                }

                latex.insert_str(0, &counters);
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
                                || id.starts_with("app:")
                                    && matches!(
                                        self.label_type,
                                        None | Some(LabelType::AppendixLabel)
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
                                    LabelType::AppendixLabel => "app",
                                    LabelType::SupplementLabel => "sup",
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
