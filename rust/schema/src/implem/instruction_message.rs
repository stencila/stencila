use crate::{Author, InstructionMessage, MessagePart, MessageRole};

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
            role: Some(MessageRole::Assistant),
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
