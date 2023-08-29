// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

/// Indicates the action (create, update or delete) to derive for a `Form`.
#[derive(Debug, Clone, PartialEq, Display, Serialize, Deserialize, SmartDefault, Strip, Read, Write, ToHtml, ToText)]
#[serde(crate = "common::serde")]
pub enum FormDeriveAction {
    #[default]
    Create,
    Update,
    Delete,
    UpdateOrDelete,
}
