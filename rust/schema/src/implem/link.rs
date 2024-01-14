use codec_losses::lost_options;

use crate::{prelude::*, Link};

impl MarkdownCodec for Link {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id, rel))
            .push_str("[")
            .push_prop_fn("content", |context| self.content.to_markdown(context))
            .push_str("](")
            .push_prop_str("target", &self.target);

        if let Some(title) = &self.title {
            context
                .push_str(" \"")
                .push_prop_fn("title", |context| title.to_markdown(context))
                .push_str("\"");
        }

        context.push_str(")").exit_node();
    }
}
