use crate::{MessagePart, Text};

impl<S> From<S> for MessagePart
where
    S: AsRef<str>,
{
    fn from(value: S) -> Self {
        Self::Text(Text::from(value))
    }
}
