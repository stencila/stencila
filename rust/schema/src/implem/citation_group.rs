use codec_info::lost_options;

use crate::{CitationGroup, prelude::*};

impl CitationGroup {
    pub fn to_jats_special(&self) -> (String, Losses) {
        (
            [
                "(",
                &self
                    .items
                    .iter()
                    .map(|item| item.to_jats_special().0)
                    .join("; "),
                ")",
            ]
            .concat(),
            Losses::none(),
        )
    }
}

impl MarkdownCodec for CitationGroup {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id))
            .push_str("[");

        for (index, item) in self.items.iter().enumerate() {
            if index > 0 {
                context.push_str("; ");
            }
            item.to_markdown(context);
        }

        context.push_str("]").exit_node();
    }
}
