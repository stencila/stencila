use crate::prelude::*;

#[derive(Debug, Default, Clone, PartialEq, Deref, DerefMut, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
pub struct Cord(String);

impl Cord {
    pub fn new<S: AsRef<str>>(value: S) -> Self {
        Self(value.as_ref().to_string())
    }
}

impl<S> From<S> for Cord
where
    S: AsRef<str>,
{
    fn from(value: S) -> Self {
        Self::new(value)
    }
}

impl From<Cord> for String {
    fn from(value: Cord) -> Self {
        value.0.clone()
    }
}
