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

impl ToText for Object {
    fn to_text(&self) -> (String, Losses) {
        let mut text = String::new();
        let mut losses = Losses::new([Loss::of_structure(LossDirection::Encode, "Object")]);

        for (name, value) in self.iter() {
            if !text.is_empty() {
                text.push(' ');
            }

            text.push_str(name);

            text.push(' ');

            let (value_text, mut value_losses) = value.to_text();
            text.push_str(&value_text);
            losses.add_all(&mut value_losses);
        }

        (text, losses)
    }
}
