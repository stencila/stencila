use crate::prelude::*;

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
pub struct TextValue(pub String);
