use crate::prelude::*;

#[derive(Debug, Default, Clone, PartialEq, Deref, DerefMut, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
pub struct Cord(pub String);

impl Cord {
    pub fn new<S: AsRef<str>>(value: S) -> Self {
        Self(value.as_ref().to_string())
    }
}
