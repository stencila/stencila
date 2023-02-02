//! Generated file, do not edit

use crate::prelude::*;

use super::node::Node;
use super::string::String;

/// A variable representing a name / value pair.
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
pub struct Variable {
    /// The type of this item
    r#type: MustBe!("Variable"),

    /// The identifier for this item
    id: String,

    /// The namespace, usually a document path, within which the variable resides
    namespace: String,

    /// The name of the variable.
    name: String,

    /// The expected type of variable e.g. `Number`, `Timestamp`, `Datatable`
    kind: Option<String>,

    /// The value of the variable.
    value: Option<Box<Node>>,
}
