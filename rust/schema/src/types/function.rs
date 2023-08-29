// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::parameter::Parameter;
use super::string::String;
use super::validator::Validator;

/// A function with a name, which might take Parameters and return a value of a certain type.
#[skip_serializing_none]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, Strip, Read, Write, ToHtml, ToText)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct Function {
    /// The type of this item
    pub r#type: MustBe!("Function"),

    /// The identifier for this item
    pub id: Option<String>,

    /// The name of the function.
    pub name: String,

    /// The parameters of the function.
    pub parameters: Vec<Parameter>,

    /// Non-core optional fields
    #[serde(flatten)]
    pub options: Box<FunctionOptions>,
}

#[skip_serializing_none]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, Strip, Read, Write, ToHtml, ToText)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct FunctionOptions {
    /// The return type of the function.
    pub returns: Option<Validator>,
}

impl Function {
    pub fn new(name: String, parameters: Vec<Parameter>) -> Self {
        Self {
            name,
            parameters,
            ..Default::default()
        }
    }
}
