use stencila_codec_info::{lost_exec_options, lost_options, lost_props};
use stencila_codec_latex_trait::{latex_to_image, to_latex};
use stencila_codec_markdown_trait::to_markdown;

use crate::{
    CodeChunk, Duration, ExecutionBounds, ExecutionMode, LabelType, MessageLevel, Timestamp,
    prelude::*,
};

use super::utils::caption_to_dom;

impl CodeChunk {
    pub fn has_warnings_errors_or_exceptions(&self) -> bool {
        self.options
            .compilation_messages
            .iter()
            .flatten()
            .any(|message| {
                matches!(
                    message.level,
                    MessageLevel::Warning | MessageLevel::Error | MessageLevel::Exception
                )
            })
            || self
                .options
                .execution_messages
                .iter()
                .flatten()
                .any(|message| {
                    matches!(
                        message.level,
                        MessageLevel::Warning | MessageLevel::Error | MessageLevel::Exception
                    )
                })
    }
}

impl DomCodec for CodeChunk {
    fn to_dom(&self, context: &mut DomEncodeContext) {
        // Custom implementation, primarily needed for encoding of different types of
        // captions before and after the outputs

        context.enter_node(self.node_type(), self.node_id());

        if let Some(execution_mode) = &self.execution_mode {
            context.push_attr("execution-mode", &execution_mode.to_string());
        }

        if let Some(execution_bounds) = &self.execution_bounds {
            context.push_attr("execution-bounds", &execution_bounds.to_string());
        }

        self.code.to_dom_attr("code", context);

        if let Some(programming_language) = &self.programming_language {
            context.push_attr("programming-language", programming_language);
        }

        if let Some(label_type) = &self.label_type {
            context.push_attr("label-type", &label_type.to_string());
        }

        if let Some(label) = &self.label {
            context.push_attr("label", label);
        }

        if let Some(label_automatically) = &self.label_automatically {
            context.push_attr("label-automatically", &label_automatically.to_string());
        }

        if let Some(is_echoed) = &self.is_echoed {
            context.push_attr("is-echoed", &is_echoed.to_string());
        }

        if let Some(is_hidden) = &self.is_hidden {
            context.push_attr("is-hidden", &is_hidden.to_string());
        }

        macro_rules! exec_option {
            ($name:literal, $prop: ident) => {
                if let Some(value) = &self.options.$prop {
                    value.to_dom_attr($name, context)
                }
            };
        }
        exec_option!("execution-count", execution_count);
        exec_option!("execution-required", execution_required);
        exec_option!("execution-status", execution_status);
        exec_option!("execution-bounded", execution_bounded);

        if let Some(value) = &self.options.execution_ended {
            Timestamp::to_dom_attr("execution-ended", value, context);
        }
        if let Some(value) = &self.options.execution_duration {
            Duration::to_dom_attr("execution-duration", value, context);
        }

        if let Some(compilation_messages) = &self.options.compilation_messages {
            context.push_slot_fn("div", "compilation-messages", |context| {
                compilation_messages.to_dom(context)
            });
        }

        if let Some(execution_messages) = &self.options.execution_messages {
            context.push_slot_fn("div", "execution-messages", |context| {
                execution_messages.to_dom(context)
            });
        }

        if let Some(authors) = &self.authors {
            context.push_slot_fn("div", "authors", |context| authors.to_dom(context));
        }

        if let Some(provenance) = &self.provenance {
            context.push_slot_fn("div", "provenance", |context| provenance.to_dom(context));
        }

        if let Some(id) = &self.id {
            context
                .enter_slot("div", "id")
                .push_attr("id", id)
                .exit_slot();
        }

        if let Some(LabelType::TableLabel) = &self.label_type {
            context.push_slot_fn("div", "caption", |context| {
                caption_to_dom(context, "table-label", "Table", &self.label, &self.caption)
            });
        }

        if let Some(outputs) = &self.outputs {
            context.push_slot_fn("div", "outputs", |context| outputs.to_dom(context));
        }

        if let Some(LabelType::FigureLabel) = &self.label_type {
            context.push_slot_fn("div", "caption", |context| {
                caption_to_dom(
                    context,
                    "figure-label",
                    "Figure",
                    &self.label,
                    &self.caption,
                )
            });
        }

        context.exit_node();
    }
}

impl LatexCodec for CodeChunk {
    fn to_latex(&self, context: &mut LatexEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, execution_mode, execution_bounds))
            .merge_losses(lost_exec_options!(self));

        if context.render {
            if let Some(outputs) = &self.outputs {
                if context.reproducible {
                    context.link_begin(None);
                }

                context.property_fn(NodeProperty::Outputs, |context| {
                    for output in outputs {
                        if context.has_format_via_pandoc() {
                            // Encode output to LaTeX
                            let (latex, ..) = to_latex(
                                output,
                                Format::Svg,
                                false,
                                true,
                                context.highlight,
                                context.reproducible,
                            );

                            if latex.trim().is_empty() {
                                continue;
                            }

                            if matches!(output, Node::ImageObject(..)) {
                                // Output already encoded as a LaTeX image in call to `to_latex`  above
                                // so just add that here
                                context.str(&latex);
                            } else {
                                // Convert the LaTeX to an image (so it can not be edited)
                                let path = context.temp_dir.join(format!("{}.svg", self.node_id()));
                                if let Err(error) = latex_to_image(&latex, &path, None) {
                                    tracing::error!(
                                        "While encoding code chunk output `{output}` to image: {error}\n\nLaTeX: {latex}"
                                    );
                                } else {
                                    let path = path.to_string_lossy();
                                    context.str(r"\includegraphics{").str(&path).char('}');
                                }
                            }
                        } else {
                            output.to_latex(context);
                        }
                    }
                });

                if context.reproducible {
                    context.link_end();
                }
            } else if context.reproducible {
                // If code chunk does not have any outputs and encoding as reproducible
                // then create a link for it
                context
                    .str("\n\n")
                    .link_with(
                        None,
                        &format!(
                            r"\verb|[Code chunk {}]|",
                            match &self.id {
                                Some(id) => id.replace('|', ""),
                                _ => String::new(),
                            }
                        ),
                    )
                    .str("\n\n");
            }

            context
                .merge_losses(lost_options!(
                    self,
                    programming_language,
                    is_echoed,
                    is_hidden
                ))
                .merge_losses(lost_props!(self, code))
                .exit_node()
                .newline();

            return;
        } else {
            context.merge_losses(lost_options!(self, outputs));
        }

        if matches!(context.format, Format::Rnw) {
            if let Some(lang) = &self.programming_language
                && lang.to_lowercase() != "r"
            {
                context.merge_losses(lost_options!(self, programming_language));
            }

            context.str("<<");
            let mut need_comma = false;

            if let Some(id) = &self.id {
                context.str(id);
                need_comma = true;
            }

            if let Some(is_echoed) = self.is_echoed {
                if need_comma {
                    context.char(',');
                }
                context
                    .str("echo=")
                    .str(&is_echoed.to_string().to_uppercase());
                need_comma = true;
            }

            if let Some(is_hidden) = self.is_hidden {
                if need_comma {
                    context.char(',');
                }
                context
                    .str("hidden=")
                    .str(&is_hidden.to_string().to_uppercase());
            }

            context
                .str(">>=")
                .newline()
                // Note: this intentionally does not escape code
                .property_str(NodeProperty::Code, &self.code);

            if !self.code.ends_with('\n') {
                context.newline();
            }

            context.char('@').newline();
        } else {
            const ENVIRON: &str = r"chunk";
            context.environ_begin(ENVIRON);

            if self.programming_language.is_some()
                || self.execution_mode.is_some()
                || self.execution_bounds.is_some()
            {
                context.char('[');

                if let Some(lang) = &self.programming_language {
                    context.property_str(NodeProperty::ProgrammingLanguage, lang);
                }

                if let Some(mode) = &self.execution_mode
                    && !matches!(mode, ExecutionMode::Need)
                {
                    context.char(',').property_str(
                        NodeProperty::ExecutionMode,
                        &mode.to_string().to_lowercase(),
                    );
                }

                if let Some(bounds) = &self.execution_bounds
                    && !matches!(bounds, ExecutionBounds::Main)
                {
                    context.char(',').property_str(
                        NodeProperty::ExecutionBounds,
                        &bounds.to_string().to_lowercase(),
                    );
                }

                if let Some(id) = &self.id {
                    context.str(",id=").property_str(NodeProperty::Id, id);
                }

                context.char(']');
            }

            context
                .newline()
                // Note: this intentionally does not escape code
                .property_str(NodeProperty::Code, &self.code);

            if !self.code.ends_with('\n') {
                context.newline();
            }

            context.environ_end(ENVIRON);
        }

        context.exit_node();

        if !context.coarse {
            context.newline();
        }
    }
}

impl MarkdownCodec for CodeChunk {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        let backticks = context.enclosing_backticks(&self.code);

        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id))
            .merge_losses(lost_exec_options!(self));

        if context.render {
            context.merge_losses(lost_props!(self, code, execution_mode, execution_bounds));

            for (index, output) in self.outputs.iter().flatten().enumerate() {
                if index > 0 {
                    context.push_str("\n\n");
                }
                output.to_markdown(context);
            }

            context.push_str("\n\n").exit_node();

            return;
        }

        if matches!(context.format, Format::Myst) {
            let is_mermaid = self.programming_language.as_deref() == Some("mermaid");

            context
                .myst_directive(
                    '`',
                    if is_mermaid { "mermaid" } else { "code-cell" },
                    |context| {
                        if let (false, Some(lang)) = (is_mermaid, &self.programming_language) {
                            context
                                .push_str(" ")
                                .push_prop_str(NodeProperty::ProgrammingLanguage, lang);
                        }
                    },
                    |context| {
                        if let Some(mode) = &self.execution_mode
                            && !matches!(mode, ExecutionMode::Need)
                        {
                            context.myst_directive_option(
                                NodeProperty::ExecutionMode,
                                Some("mode"),
                                &mode.to_string().to_lowercase(),
                            );
                        }

                        if let Some(bounds) = &self.execution_bounds
                            && !matches!(bounds, ExecutionBounds::Main)
                        {
                            context.myst_directive_option(
                                NodeProperty::ExecutionBounds,
                                Some("bounds"),
                                &bounds.to_string().to_lowercase(),
                            );
                        }

                        if matches!(self.is_echoed, Some(true)) {
                            context.myst_directive_option(
                                NodeProperty::IsEchoed,
                                Some("echo"),
                                "true",
                            );
                        }

                        if matches!(self.is_hidden, Some(true)) {
                            context.myst_directive_option(
                                NodeProperty::IsHidden,
                                Some("hide"),
                                "true",
                            );
                        }

                        if let Some(label_type) = &self.label_type {
                            context.myst_directive_option(
                                NodeProperty::LabelType,
                                Some("type"),
                                match label_type {
                                    LabelType::FigureLabel => "figure",
                                    LabelType::TableLabel => "table",
                                    // Should be unreachable, but in case it is reached..
                                    _ => "chunk",
                                },
                            );
                        }

                        if let Some(label) = &self.label {
                            context.myst_directive_option(NodeProperty::Label, None, label);
                        }

                        if let Some(caption) = &self.caption {
                            // Note: caption must be a single line
                            let caption = to_markdown(caption).replace('\n', " ");
                            context.myst_directive_option(NodeProperty::Caption, None, &caption);
                        }
                    },
                    |context| {
                        context.push_prop_fn(NodeProperty::Code, |context| {
                            self.code.to_markdown(context);
                            if !self.code.ends_with('\n') {
                                context.newline();
                            }
                        });
                    },
                )
                .exit_node()
                .newline();
        } else if matches!(context.format, Format::Qmd) {
            let lang = self.programming_language.clone().unwrap_or_default();

            context
                .push_str("```{")
                .push_prop_str(NodeProperty::ProgrammingLanguage, &lang)
                .push_str("}\n");

            let comment = if lang.ends_with("js") { "//| " } else { "#| " };
            let mut has_comments = false;

            if !self.label_automatically.unwrap_or(true)
                && let Some(label) = &self.label
            {
                context
                    .push_str(comment)
                    .push_str("label: ")
                    .push_prop_str(NodeProperty::Label, label)
                    .push_str("\n");
                has_comments = true;
            }

            if let Some(caption) = &self.caption {
                context
                    .push_str(comment)
                    .push_str(match &self.label_type {
                        Some(LabelType::TableLabel) => "tbl",
                        _ => "fig",
                    })
                    .push_str("-cap: \"")
                    .push_prop_str(
                        NodeProperty::Caption,
                        &to_markdown(caption).replace('\n', " "),
                    )
                    .push_str("\"\n");
                has_comments = true;
            }

            if has_comments {
                context.newline();
            }

            context
                .push_prop_fn(NodeProperty::Code, |context| {
                    self.code.to_markdown(context);
                    if !self.code.ends_with('\n') {
                        context.newline();
                    }
                })
                .push_str("```\n\n")
                .exit_node()
                .newline();
        } else {
            let wrapped =
                if self.label_type.is_some() || self.label.is_some() || self.caption.is_some() {
                    context.push_colons();

                    if let Some(label_type) = &self.label_type {
                        context.push_str(match label_type {
                            LabelType::FigureLabel => " figure",
                            LabelType::TableLabel => " table",
                            // Should be unreachable, but in case it is reached..
                            _ => " chunk",
                        });
                    } else {
                        context.push_str(" chunk");
                    }

                    if !self.label_automatically.unwrap_or(true)
                        && let Some(label) = &self.label
                    {
                        context.push_str(" ");
                        context.push_prop_str(NodeProperty::Label, label);
                    }

                    context.push_str("\n\n");

                    true
                } else {
                    false
                };

            if let Some(caption) = &self.caption {
                context.push_prop_fn(NodeProperty::Caption, |context| {
                    caption.to_markdown(context)
                });
            }

            if !wrapped
                && matches!(self.programming_language.as_deref(), Some("docsql"))
                && !self.code.contains(['\n', ';'])
                && !self.code.contains("let ")
            {
                context
                    .push_str("{{")
                    .push_prop_str(NodeProperty::Code, &self.code)
                    .push_str("}}\n");
            } else {
                context.push_str(&backticks);

                if let Some(lang) = &self.programming_language {
                    context
                        .push_prop_str(NodeProperty::ProgrammingLanguage, lang)
                        .push_str(" ");
                }

                context.push_str("exec");

                if let Some(mode) = &self.execution_mode
                    && !matches!(mode, ExecutionMode::Need)
                {
                    context.push_str(" ").push_prop_str(
                        NodeProperty::ExecutionMode,
                        &mode.to_string().to_lowercase(),
                    );
                }

                if let Some(bounds) = &self.execution_bounds
                    && !matches!(bounds, ExecutionBounds::Main)
                {
                    context.push_str(" ").push_prop_str(
                        NodeProperty::ExecutionBounds,
                        &bounds.to_string().to_lowercase(),
                    );
                }

                if matches!(self.is_echoed, Some(true)) {
                    context
                        .push_str(" ")
                        .push_prop_str(NodeProperty::IsEchoed, "echo");
                }

                if matches!(self.is_hidden, Some(true)) {
                    context
                        .push_str(" ")
                        .push_prop_str(NodeProperty::IsHidden, "hide");
                }

                context
                    .newline()
                    .push_prop_fn(NodeProperty::Code, |context| self.code.to_markdown(context));

                if !self.code.ends_with('\n') {
                    context.newline();
                }

                context.push_str(&backticks).newline();
            }

            if wrapped {
                context.newline().push_colons().newline();
            }

            if matches!(context.format, Format::Llmd)
                && !self.is_hidden.unwrap_or_default()
                && !self
                    .outputs
                    .as_ref()
                    .map(|outputs| outputs.is_empty())
                    .unwrap_or(true)
            {
                // Encode outputs as separate paragraphs (ensuring blank line after each)
                context.push_str("\n=>\n\n");
                for output in self.outputs.iter().flatten() {
                    output.to_markdown(context);
                    if !context.content.ends_with("\n\n") {
                        context.push_str("\n\n");
                    }
                }
            } else {
                context.merge_losses(lost_options!(self, outputs));
            }

            context.exit_node().newline();
        }
    }
}

impl TextCodec for CodeChunk {
    fn to_text(&self) -> String {
        // If any outputs then render those to text (i.e. similar to render mode for Markdown),
        // otherwise, encode the code.
        let content = if let Some(outputs) = &self.outputs {
            outputs.to_text()
        } else {
            self.code.to_text()
        };

        [&content, "\n\n"].concat()
    }
}
