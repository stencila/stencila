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
        Self::new(Cord::new(value))
    }
}

impl DomCodec for Text {
    fn to_dom(&self, context: &mut DomEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .push_text(&self.to_value_string())
            .exit_elem();
    }
}

impl MarkdownCodec for Text {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        // To avoid unnecessary, redundant entries for `Text.value` in `Mapping`
        // this custom implementation just pushes the string.
        context
            .enter_node(self.node_type(), self.node_id())
            .push_str(&self.value)
            .exit_node();
    }
}
