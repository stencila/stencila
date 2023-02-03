//! Generated file, do not edit

use crate::prelude::*;

use super::node::Node;
use super::string::String;

/// A variable representing a name / value pair.
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Reconcile, Hydrate)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct Variable {
    /// The type of this item
    #[autosurgeon(with = "autosurgeon_must_be")]
    pub r#type: MustBe!("Variable"),

    /// The identifier for this item
    #[key]
    pub id: Option<String>,

    /// The namespace, usually a document path, within which the variable resides
    pub namespace: String,

    /// The name of the variable.
    pub name: String,

    /// The expected type of variable e.g. `Number`, `Timestamp`, `Datatable`
    pub kind: Option<String>,

    /// The value of the variable.
    pub value: Option<Box<Node>>,
}
