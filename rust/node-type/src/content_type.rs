use common::{
    serde::{Deserialize, Serialize},
    strum::{Display, EnumString},
};

/// The type of content
#[derive(Default, Debug, Display, EnumString, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case", crate = "common::serde")]
#[strum(ascii_case_insensitive, crate = "common::strum")]
pub enum ContentType {
    #[default]
    Block,
    Inline,
}
