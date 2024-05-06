use crate::{prelude::*, Cord, Text};

impl Text {
    pub fn to_value_string(&self) -> String {
        self.value.to_string()
    }

    pub fn to_jats_special(&self) -> (String, Losses) {
        use codec_jats_trait::encode::escape;

        (escape(self.value.as_str()), Losses::none())
    }
}

impl<S> From<S> for Text
where
    S: AsRef<str>,
{
    fn from(value: S) -> Self {
        Self::new(Cord::from(value))
    }
}

impl DomCodec for Text {
    fn to_dom(&self, context: &mut DomEncodeContext) {
        context.enter_node(self.node_type(), self.node_id());
        self.value.to_dom(context);
        context.exit_elem();
    }
}

impl MarkdownCodec for Text {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .push_prop_fn(NodeProperty::Value, |context| {
                self.value.to_markdown(context)
            })
            .exit_node();
    }
}
