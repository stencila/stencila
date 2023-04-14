use crate::prelude::*;

/// Indicates the action (create, update or delete) to derive for a `Form`.
#[derive(Debug, Clone, PartialEq, Display, Serialize, Deserialize, Defaults, Read, Write, ToHtml)]
#[serde(untagged, crate = "common::serde")]
#[def = "Create"]
pub enum FormDeriveAction {
    Create,
    Update,
    Delete,
    UpdateOrDelete,
}
