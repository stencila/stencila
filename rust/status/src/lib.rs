use common::{
    clap::{self, ValueEnum},
    serde::Serialize,
    strum::{Display, EnumString},
};

#[derive(Debug, Display, Clone, Copy, ValueEnum, EnumString, Serialize)]
#[strum(serialize_all = "kebab-case", crate = "common::strum")]
#[serde(rename_all = "kebab-case", crate = "common::serde")]
pub enum Status {
    UnderDevelopment,
    Alpha,
    Beta,
    Unstable,
    Stable,
}
