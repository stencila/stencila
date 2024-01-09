use crate::{Message, MessagePart};

impl<S> From<S> for Message
where
    S: AsRef<str>,
{
    fn from(value: S) -> Self {
        Self::new(vec![MessagePart::from(value)])
    }
}
