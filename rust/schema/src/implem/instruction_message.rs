use stencila_codec_info::lost_options;

use crate::{Author, Inline, InstructionMessage, MessageRole, Text, prelude::*};

impl InstructionMessage {
    pub fn system<S: AsRef<str>>(value: S, authors: Option<Vec<Author>>) -> Self {
        Self {
            role: Some(MessageRole::System),
            content: vec![Inline::Text(Text::from(value.as_ref()))],
            authors,
            ..Default::default()
        }
    }

    pub fn user<S: AsRef<str>>(value: S, authors: Option<Vec<Author>>) -> Self {
        Self {
            role: Some(MessageRole::User),
            content: vec![Inline::Text(Text::from(value.as_ref()))],
            authors,
            ..Default::default()
        }
    }

    pub fn assistant<S: AsRef<str>>(value: S, authors: Option<Vec<Author>>) -> Self {
        Self {
            role: Some(MessageRole::Model),
            content: vec![Inline::Text(Text::from(value.as_ref()))],
            authors,
            ..Default::default()
        }
    }
}

impl<S> From<S> for InstructionMessage
where
    S: AsRef<str>,
{
    fn from(value: S) -> Self {
        Self {
            content: vec![Inline::Text(Text::from(value.as_ref()))],
            ..Default::default()
        }
    }
}

impl MarkdownCodec for InstructionMessage {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id, role, authors, provenance))
            .push_prop_fn(NodeProperty::Content, |context| {
                self.content.to_markdown(context)
            })
            .exit_node();
    }
}
