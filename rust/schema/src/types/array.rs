use common::derive_more::{Deref, DerefMut};

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

impl ToText for Array {
    fn to_text(&self) -> (String, Losses) {
        let mut text = String::new();
        let mut losses = Losses::none();

        for (index, item) in self.iter().enumerate() {
            if index != 0 {
                text.push(' ');
            }

            let (item_text, mut item_losses) = item.to_text();
            text.push_str(&item_text);
            losses.add_all(&mut item_losses);
        }

        (text, losses)
    }
}
