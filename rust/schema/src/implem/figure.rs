use codec_info::{lost_options, lost_work_options};

use crate::{prelude::*, Figure};

impl MarkdownCodec for Figure {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id))
            .merge_losses(lost_work_options!(self))
            .push_semis()
            .push_str(" figure");

        if let Some(label) = &self.label {
            context.push_str(" ");
            context.push_prop_str(NodeProperty::Label, label);
        }

        context.push_str("\n\n").increase_depth();

        if let Some(caption) = &self.caption {
            context.push_prop_fn(NodeProperty::Caption, |context| {
                caption.to_markdown(context)
            });
        }

        context
            .push_prop_fn(NodeProperty::Content, |context| {
                self.content.to_markdown(context)
            })
            .decrease_depth()
            .push_semis()
            .newline()
            .exit_node()
            .newline();
    }
}
