//! Generated file, do not edit

use crate::prelude::*;

use super::parameter::Parameter;
use super::string::String;
use super::validator::Validator;

/// A function with a name, which might take Parameters and return a value of a certain type.
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Reconcile, Hydrate)]
#[serde(crate = "common::serde")]
pub struct Function {
    /// The type of this item
    #[autosurgeon(with = "autosurgeon_must_be")]
    r#type: MustBe!("Function"),

    /// The identifier for this item
    id: Option<String>,

    /// The name of the function.
    name: String,

    /// The parameters of the function.
    parameters: Vec<Parameter>,

    /// Non-core optional fields
    #[serde(flatten)]
    options: Box<FunctionOptions>,
}

#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Reconcile, Hydrate)]
#[serde(crate = "common::serde")]
pub struct FunctionOptions {
    /// The return type of the function.
    returns: Option<Validator>,
}
