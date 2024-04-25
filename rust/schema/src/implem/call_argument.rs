use codec_info::{lost_exec_options, lost_options};
use codec_json5_trait::Json5Codec;

use crate::{prelude::*, CallArgument};

impl MarkdownCodec for CallArgument {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id, programming_language))
            .merge_losses(lost_exec_options!(self))
            .push_prop_str(NodeProperty::Name, &self.name)
            .push_str("=");

        if self.code.is_empty() && self.value.is_some() {
            let json5 = self
                .value
                .as_ref()
                .expect("should be some")
                .to_json5()
                .unwrap_or_default();
            context.push_prop_str(NodeProperty::Value, &json5);
        } else {
            context
                .push_str("`")
                .push_prop_str(NodeProperty::Code, &self.code)
                .push_str("`");
        };

        context.exit_node();
    }
}
