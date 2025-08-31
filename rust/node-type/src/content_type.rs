use serde::{Deserialize, Serialize};

use common::strum::{Display, EnumString};

/// The type of content
#[derive(Default, Debug, Display, EnumString, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
#[strum(ascii_case_insensitive, crate = "common::strum")]
pub enum ContentType {
    #[default]
    Block,
    Inline,
}
