use codec_losses::{lost_options, lost_work_options};

use crate::{prelude::*, Figure};

impl MarkdownCodec for Figure {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id))
            .merge_losses(lost_work_options!(self));

        let fence = ":".repeat(3 + context.depth * 2);

        context.push_str(&fence).push_str(" figure");

        if let Some(label) = &self.label {
            context.push_str(" ");
            context.push_prop_str("label", label);
        }

        context.push_str("\n\n").increase_depth();

        if let Some(caption) = &self.caption {
            context.push_prop_fn("caption", |context| caption.to_markdown(context));
        }

        context.push_prop_fn("content", |context| self.content.to_markdown(context));

        context
            .decrease_depth()
            .push_str(&fence)
            .newline()
            .exit_node()
            .newline();
    }
}
