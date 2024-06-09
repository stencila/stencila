use codec_info::{lost_exec_options, lost_options};

use crate::{prelude::*, CodeChunk, Duration, LabelType, Timestamp};

use super::utils::caption_to_dom;

impl DomCodec for CodeChunk {
    fn to_dom(&self, context: &mut DomEncodeContext) {
        // Custom implementation, primarily needed for encoding of different types of
        // captions before and after the outputs

        context.enter_node(self.node_type(), self.node_id());

        if let Some(auto_exec) = &self.auto_exec {
            context.push_attr("auto-exec", &auto_exec.to_string());
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

        if let (Some(LabelType::TableLabel), Some(caption)) = (&self.label_type, &self.caption) {
            context.push_slot_fn("div", "caption", |context| {
                caption_to_dom(context, "table-label", "Table", &self.label, caption)
            });
        }

        if let Some(outputs) = &self.outputs {
            context.push_slot_fn("div", "outputs", |context| outputs.to_dom(context));
        }

        if let (Some(LabelType::FigureLabel), Some(caption)) = (&self.label_type, &self.caption) {
            context.push_slot_fn("div", "caption", |context| {
                caption_to_dom(context, "figure-label", "Figure", &self.label, caption)
            });
        }

        context.exit_node();
    }
}

impl MarkdownCodec for CodeChunk {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        if context.render {
            // Encode outputs as separate paragraphs
            for output in self.outputs.iter().flatten() {
                output.to_markdown(context);
                if !context.content.ends_with("\n\n") {
                    context.push_str("\n\n");
                }
            }
            return;
        }

        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id, outputs))
            .merge_losses(lost_exec_options!(self));

        let wrapped = if self.label_type.is_some() || self.label.is_some() || self.caption.is_some()
        {
            context.push_semis();

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

        if let Some(auto) = &self.auto_exec {
            context
                .push_str(" auto=")
                .push_prop_str(NodeProperty::AutoExec, &auto.to_string().to_lowercase());
        }

        context
            .newline()
            .push_prop_fn(NodeProperty::Code, |context| self.code.to_markdown(context));

        if !self.code.ends_with('\n') {
            context.newline();
        }

        context.push_str("```\n");

        if wrapped {
            context.newline().push_semis().newline();
        }

        context.exit_node().newline();
    }
}
