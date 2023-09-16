use common::{
    clap::{self, ValueEnum},
    serde::{Deserialize, Serialize},
    strum::{Display, EnumString},
};
use schemars::JsonSchema;

#[derive(
    Debug, Display, Default, Clone, Copy, ValueEnum, EnumString, Deserialize, Serialize, JsonSchema,
)]
#[strum(serialize_all = "kebab-case", crate = "common::strum")]
#[serde(rename_all = "kebab-case", crate = "common::serde")]
pub enum Status {
    Planned,
    Experimental,
    UnderDevelopment,
    Alpha,
    Beta,
    #[default]
    Stable,
}

impl Status {
    /// Whether the status is the default
    pub fn is_default(&self) -> bool {
        matches!(self, Self::Stable)
    }

    /// Get the emoji associated with the status
    pub fn emoji(&self) -> &str {
        use Status::*;
        match self {
            Planned => "🧭",
            Experimental => "🧪",
            UnderDevelopment => "🚧",
            Alpha => "🟥",
            Beta => "🔶",
            Stable => "🟢",
        }
    }
}
