use crate::prelude::*;

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
pub struct TextValue(pub String);

impl ToText for TextValue {
    fn to_text(&self) -> (String, Losses) {
        (self.0.to_string(), Losses::none())
    }
}
