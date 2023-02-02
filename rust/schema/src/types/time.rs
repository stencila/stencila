//! Generated file, do not edit

use crate::prelude::*;

use super::string::String;

/// A point in time recurring on multiple days
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Reconcile, Hydrate)]
#[serde(crate = "common::serde")]
pub struct Time {
    /// The time of day as a string in format `hh:mm:ss[Z|(+|-)hh:mm]`.
    value: String,
}
