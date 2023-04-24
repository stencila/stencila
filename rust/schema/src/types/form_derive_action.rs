use crate::prelude::*;

/// Indicates the action (create, update or delete) to derive for a `Form`.
#[derive(Debug, Clone, PartialEq, Display, Serialize, Deserialize, Defaults, Strip, Read, Write, ToHtml)]
#[serde(crate = "common::serde")]
#[def = "Create"]
pub enum FormDeriveAction {
    Create,
    Update,
    Delete,
    UpdateOrDelete,
}
