use codec_losses::lost_options;

use crate::{prelude::*, ModifyInline, ModifyOperation};

impl MarkdownCodec for ModifyInline {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id));

        context
            .push_str("{!!")
            .push_prop_fn("content", |context| self.content.to_markdown(context));

        let modified =
            ModifyOperation::apply_many(&self.operations, &self.content).unwrap_or_default();
        context
            .push_str("!>")
            .push_prop_fn("operations", |context| modified.to_markdown(context));

        context.push_str("!!}").exit_node();
    }
}
