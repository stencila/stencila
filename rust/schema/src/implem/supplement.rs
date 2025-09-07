use stencila_codec_info::lost_options;

use crate::{Supplement, prelude::*};

use super::utils::caption_to_dom;

impl DomCodec for Supplement {
    fn to_dom(&self, context: &mut DomEncodeContext) {
        context.enter_node(self.node_type(), self.node_id());

        if let Some(work_type) = &self.work_type {
            context.push_attr("work-type", &work_type.to_string());
        }

        if let Some(label) = &self.label {
            context.push_attr("label", label);
        }

        if let Some(label_automatically) = &self.label_automatically {
            context.push_attr("label-automatically", &label_automatically.to_string());
        }

        if let Some(target) = &self.target {
            context.push_attr("target", target);
        }

        if let Some(id) = &self.id {
            context
                .enter_slot("div", "id")
                .push_attr("id", id)
                .exit_slot();
        }

        if let Some(messages) = &self.options.compilation_messages {
            context.push_slot_fn("div", "compilation-messages", |context| {
                messages.to_dom(context)
            });
        }

        if self.label.is_some() || self.caption.is_some() {
            context.push_slot_fn("div", "caption", |context| {
                caption_to_dom(
                    context,
                    "supplement-label",
                    "Supplement",
                    &self.label,
                    &self.caption,
                )
            });
        }

        if let Some(work) = &self.options.work {
            context.push_slot_fn("div", "work", |context| work.to_dom(context));
        }

        context.exit_node();
    }
}

impl MarkdownCodec for Supplement {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id));

        if context.render || !matches!(context.format, Format::Smd) {
            // Just encode the label (with url if any) caption

            let label = self.label.as_deref().unwrap_or("Supplement");
            if let Some(target) = &self.target {
                context
                    .push_str("[")
                    .push_str(label)
                    .push_str("](")
                    .push_str(target)
                    .push_str(")");
            } else {
                context.push_str(label);
            }

            if let Some(caption) = &self.caption {
                context
                    .push_str(": ")
                    .push_prop_fn(NodeProperty::Caption, |context| {
                        caption.to_markdown(context)
                    });
            } else {
                context.push_str("\n\n");
            }
        } else {
            context.push_colons().push_str(" supplement");

            if let Some(label) = &self.label {
                context
                    .push_str(" ")
                    .push_prop_str(NodeProperty::Label, label);
            }

            context.push_str("\n\n");

            if let Some(caption) = &self.caption {
                context
                    .increase_depth()
                    .push_prop_fn(NodeProperty::Caption, |context| {
                        caption.to_markdown(context)
                    })
                    .decrease_depth();
            }

            if let Some(target) = &self.target {
                context.push_str("[](").push_str(target).push_str(")\n\n");
            }

            context.push_colons().push_str("\n\n");
        }

        context.exit_node();
    }
}
