use crate::prelude::*;

use super::primitive::Primitive;

#[derive(Debug, Default, Clone, PartialEq, Deref, DerefMut, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
pub struct Array(pub Vec<Primitive>);

impl Array {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<const N: usize> From<[Primitive; N]> for Array {
    fn from(value: [Primitive; N]) -> Self {
        Array(Vec::from(value))
    }
}
