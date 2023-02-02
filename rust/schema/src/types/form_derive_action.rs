//! Generated file, do not edit

use crate::prelude::*;

/// Indicates the action (create, update or delete) to derive for a `Form`.
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Reconcile, Hydrate)]
#[serde(crate = "common::serde")]
#[def = "Create"]
pub enum FormDeriveAction {
    Create,
    Update,
    Delete,
    UpdateOrDelete,
}
