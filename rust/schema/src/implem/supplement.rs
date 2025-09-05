use stencila_codec_info::lost_options;

use crate::{Supplement, prelude::*};

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
                    .push_str(&target)
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
                context.push_str("[](").push_str(&target).push_str(")\n\n");
            }

            context.push_colons().push_str("\n\n");
        }

        context.exit_node();
    }
}
