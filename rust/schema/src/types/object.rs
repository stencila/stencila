use common::{
    derive_more::{Deref, DerefMut},
    indexmap::IndexMap,
};

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
