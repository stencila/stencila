use codec_info::lost_options;

use crate::{prelude::*, Author, InstructionMessage, MessagePart, MessageRole};

impl InstructionMessage {
    pub fn system<S: AsRef<str>>(value: S, authors: Option<Vec<Author>>) -> Self {
        Self {
            role: Some(MessageRole::System),
            parts: vec![MessagePart::from(value)],
            authors,
            ..Default::default()
        }
    }

    pub fn user<S: AsRef<str>>(value: S, authors: Option<Vec<Author>>) -> Self {
        Self {
            role: Some(MessageRole::User),
            parts: vec![MessagePart::from(value)],
            authors,
            ..Default::default()
        }
    }

    pub fn assistant<S: AsRef<str>>(value: S, authors: Option<Vec<Author>>) -> Self {
        Self {
            role: Some(MessageRole::Model),
            parts: vec![MessagePart::from(value)],
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
        Self::new(vec![MessagePart::from(value)])
    }
}

impl MarkdownCodec for InstructionMessage {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id, role, authors, provenance))
            .push_prop_fn(NodeProperty::Parts, |context| {
                self.parts.to_markdown(context)
            })
            .exit_node();
    }
}
