use serde::{Deserialize, Serialize};

use strum::{Display, EnumString};

/// The type of content
#[derive(Default, Debug, Display, EnumString, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
#[strum(ascii_case_insensitive)]
pub enum ContentType {
    #[default]
    Block,
    Inline,
}
