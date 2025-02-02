use common::indexmap::IndexMap;

use crate::prelude::*;

use super::primitive::Primitive;

#[derive(Debug, Default, Clone, PartialEq, Deref, DerefMut, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
pub struct Object(pub IndexMap<String, Primitive>);

impl Object {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<const N: usize> From<[(String, Primitive); N]> for Object {
    fn from(value: [(String, Primitive); N]) -> Self {
        Object(IndexMap::from(value))
    }
}

impl<const N: usize> From<[(&str, Primitive); N]> for Object {
    fn from(value: [(&str, Primitive); N]) -> Self {
        Object(IndexMap::from_iter(
            value
                .into_iter()
                .map(|(key, value)| (key.to_string(), value)),
        ))
    }
}
