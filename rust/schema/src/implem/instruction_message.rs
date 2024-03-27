use crate::{InstructionMessage, MessagePart};

impl<S> From<S> for InstructionMessage
where
    S: AsRef<str>,
{
    fn from(value: S) -> Self {
        InstructionMessage::new(vec![MessagePart::from(value)])
    }
}
