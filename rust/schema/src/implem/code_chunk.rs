use codec_info::{lost_exec_options, lost_options};
use codec_markdown_trait::to_markdown;

use crate::{prelude::*, CodeChunk, Duration, LabelType, Timestamp};

use super::utils::caption_to_dom;

impl DomCodec for CodeChunk {
    fn to_dom(&self, context: &mut DomEncodeContext) {
        // Custom implementation, primarily needed for encoding of different types of
        // captions before and after the outputs

        context.enter_node(self.node_type(), self.node_id());

        if let Some(execution_mode) = &self.execution_mode {
            context.push_attr("execution-mode", &execution_mode.to_string());
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

        if let Some(is_invisible) = &self.is_invisible {
            context.push_attr("is-invisible", &is_invisible.to_string());
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
        exec_option!("execution-kind", execution_kind);

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

impl MarkdownCodec for CodeChunk {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        if context.render || matches!(context.format, Format::Llmd) {
            // Record any execution messages
            if let Some(messages) = &self.options.execution_messages {
                for message in messages {
                    context.add_message(
                        self.node_type(),
                        self.node_id(),
                        message.level.clone().into(),
                        message.message.to_string(),
                    );
                }
            }

            // Encode label and caption (ensuring blank line after)
            if let Some(label_type) = &self.label_type {
                context.push_str(match label_type {
                    LabelType::FigureLabel => "Figure ",
                    LabelType::TableLabel => "Table ",
                });
            }
            if let Some(label) = &self.label {
                context.push_str(label).push_str(": ");
            }
            if let Some(caption) = &self.caption {
                caption.to_markdown(context)
            }
            if !context.content.ends_with("\n\n") {
                context.push_str("\n\n");
            }

            // If encoding to LLMd, encode the code (with lang and `exec` keyword)
            // but not with execution mode etc)
            if matches!(context.format, Format::Llmd) {
                context.push_str("```");

                if let Some(lang) = &self.programming_language {
                    context.push_str(lang).push_str(" ");
                }

                context.push_str("exec").newline().push_str(&self.code);

                if !self.code.ends_with('\n') {
                    context.newline();
                }

                context.push_str("```\n\n");
            }

            // Encode outputs as separate paragraphs (ensuring blank line after each)
            // (unless invisible)
            if !matches!(self.is_invisible, Some(true)) {
                for output in self.outputs.iter().flatten() {
                    output.to_markdown(context);
                    if !context.content.ends_with("\n\n") {
                        context.push_str("\n\n");
                    }
                }
            }

            return;
        }

        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id, outputs))
            .merge_losses(lost_exec_options!(self));

        if matches!(context.format, Format::Myst) {
            context.myst_directive(
                '`',
                "code-cell",
                |context| {
                    if let Some(lang) = &self.programming_language {
                        context
                            .push_str(" ")
                            .push_prop_str(NodeProperty::ProgrammingLanguage, lang);
                    }
                },
                |context| {
                    if matches!(self.is_invisible, Some(true)) {
                        context.myst_directive_option(
                            NodeProperty::IsInvisible,
                            Some("invisible"),
                            "true",
                        );
                    }

                    if let Some(execution_mode) = &self.execution_mode {
                        context.myst_directive_option(
                            NodeProperty::ExecutionMode,
                            Some("mode"),
                            &execution_mode.to_string().to_lowercase(),
                        );
                    }

                    if let Some(label_type) = &self.label_type {
                        context.myst_directive_option(
                            NodeProperty::LabelType,
                            Some("type"),
                            match label_type {
                                LabelType::FigureLabel => "figure",
                                LabelType::TableLabel => "table",
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
            );
        } else {
            let wrapped =
                if self.label_type.is_some() || self.label.is_some() || self.caption.is_some() {
                    context.push_colons();

                    if let Some(label_type) = &self.label_type {
                        context.push_str(match label_type {
                            LabelType::FigureLabel => " figure",
                            LabelType::TableLabel => " table",
                        });
                    } else {
                        context.push_str(" chunk");
                    }

                    if !self.label_automatically.unwrap_or(true) {
                        if let Some(label) = &self.label {
                            context.push_str(" ");
                            context.push_prop_str(NodeProperty::Label, label);
                        }
                    }

                    context.push_str("\n\n");

                    true
                } else {
                    false
                };

            if let Some(caption) = &self.caption {
                context
                    .increase_depth()
                    .push_prop_fn(NodeProperty::Caption, |context| {
                        caption.to_markdown(context)
                    })
                    .decrease_depth();
            }

            context.push_str("```");

            if let Some(lang) = &self.programming_language {
                context
                    .push_prop_str(NodeProperty::ProgrammingLanguage, lang)
                    .push_str(" ");
            }

            context.push_str("exec");

            if matches!(self.is_invisible, Some(true)) {
                context
                    .push_str(" ")
                    .push_prop_str(NodeProperty::IsInvisible, "invisible");
            }

            if let Some(mode) = &self.execution_mode {
                context.push_str(" ").push_prop_str(
                    NodeProperty::ExecutionMode,
                    &mode.to_string().to_lowercase(),
                );
            }

            context
                .newline()
                .push_prop_fn(NodeProperty::Code, |context| self.code.to_markdown(context));

            if !self.code.ends_with('\n') {
                context.newline();
            }

            context.push_str("```\n");

            if wrapped {
                context.newline().push_colons().newline();
            }
        }

        context.exit_node().newline();
    }
}
