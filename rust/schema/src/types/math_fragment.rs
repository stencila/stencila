//! Generated file, do not edit

use crate::prelude::*;

use super::execution_digest::ExecutionDigest;
use super::string::String;

/// A fragment of math, e.g a variable name, to be treated as inline content.
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Reconcile, Hydrate)]
#[serde(crate = "common::serde")]
pub struct MathFragment {
    /// The type of this item
    #[autosurgeon(with = "autosurgeon_must_be")]
    r#type: MustBe!("MathFragment"),

    /// The identifier for this item
    id: Option<String>,

    /// The language used for the equation e.g tex, mathml, asciimath.
    math_language: String,

    /// The code of the equation in the `mathLanguage`.
    code: String,

    /// A digest of the `code` and `mathLanguage` used to avoid unnecessary transpilation to MathML
    compile_digest: Option<ExecutionDigest>,

    /// Errors that occurred when parsing the math equation.
    errors: Option<Vec<String>>,

    /// The MathML transpiled from the `code`
    mathml: Option<String>,
}
