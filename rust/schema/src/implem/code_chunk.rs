use codec_losses::{lost_exec_options, lost_options};

use crate::{prelude::*, CodeChunk, LabelType};

impl MarkdownCodec for CodeChunk {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id, outputs))
            .merge_losses(lost_exec_options!(self));

        let fence = ":".repeat(3 + context.depth * 2);

        let wrapped = if self.label_type.is_some() || self.label.is_some() || self.caption.is_some()
        {
            context.push_str(&fence);

            if let Some(label_type) = &self.label_type {
                context.push_str(match label_type {
                    LabelType::FigureLabel => " figure",
                    LabelType::TableLabel => " table",
                });
            } else {
                context.push_str(" chunk");
            }

            if let Some(label) = &self.label {
                context.push_str(" ");
                context.push_prop_str("label", label);
            }

            context.push_str("\n\n");

            true
        } else {
            false
        };

        if let Some(caption) = &self.caption {
            context
                .increase_depth()
                .push_prop_fn("caption", |context| caption.to_markdown(context))
                .decrease_depth();
        }

        context.push_str("```");

        if let Some(lang) = &self.programming_language {
            context
                .push_prop_str("programming_language", lang)
                .push_str(" ");
        }

        context.push_str("exec");

        if let Some(auto) = &self.auto_exec {
            context
                .push_str(" auto=")
                .push_prop_str("auto_exec", &auto.to_string().to_lowercase());
        }

        context.push_str("\n").push_prop_str("code", &self.code);

        if !self.code.ends_with('\n') {
            context.push_str("\n");
        }

        context.push_str("```\n\n");

        if wrapped {
            context.push_str(&fence).push_str("\n\n");
        }

        context.exit_node();
    }
}
