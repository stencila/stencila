use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

use clap::ValueEnum;

#[derive(
    Debug, Display, Default, Clone, Copy, ValueEnum, EnumString, Deserialize, Serialize, JsonSchema,
)]
#[strum(serialize_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
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
            Alpha => "⚠️",
            Beta => "🔶",
            Stable => "🟢",
        }
    }
}
