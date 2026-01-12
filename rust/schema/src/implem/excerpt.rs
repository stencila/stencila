use stencila_codec_info::lost_options;

use crate::{Excerpt, prelude::*};

impl MarkdownCodec for Excerpt {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context.enter_node(self.node_type(), self.node_id());

        // If not rendering and format is Stencila or LLM Markdown then wrap the excerpt in ::: excerpt.
        // For LLMs this is important for citation of excerpts.
        let wrap = !context.render && matches!(context.format, Format::Smd | Format::Llmd);
        if wrap {
            context.push_colons().push_str(" excerpt");

            if let Some(id) = &self.id {
                context.push_str(" ").push_str(id);
            }

            context.push_str("\n\n").increase_depth();
        } else {
            context.merge_losses(lost_options!(self, id));
        }

        context.push_prop_fn(NodeProperty::Content, |context| {
            self.content.to_markdown(context)
        });

        if wrap {
            context
                .decrease_depth()
                .push_colons()
                .newline()
                .exit_node()
                .newline();
        } else {
            context.exit_node();
        }
    }
}
