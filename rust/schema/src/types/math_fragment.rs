// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::cord::Cord;
use super::execution_digest::ExecutionDigest;
use super::string::String;

/// A fragment of math, e.g a variable name, to be treated as inline content.
#[skip_serializing_none]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec, ReadNode, WriteNode)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[cfg_attr(feature = "proptest", derive(Arbitrary))]
#[html(elem = "math", custom)]
#[jats(elem = "inline-formula", special)]
#[markdown(special)]
pub struct MathFragment {
    /// The type of this item.
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    pub r#type: MustBe!("MathFragment"),

    /// The identifier for this item.
    #[strip(id)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The language used for the equation e.g tex, mathml, asciimath.
    #[cfg_attr(feature = "proptest-min", proptest(value = r#"String::from("lang")"#))]
    #[cfg_attr(feature = "proptest-low", proptest(regex = r#"(asciimath)|(mathml)|(tex)"#))]
    #[cfg_attr(feature = "proptest-high", proptest(regex = r#"[a-zA-Z0-9]{1,10}"#))]
    #[cfg_attr(feature = "proptest-max", proptest(strategy = r#"String::arbitrary()"#))]
    pub math_language: String,

    /// The code of the equation in the `mathLanguage`.
    #[cfg_attr(feature = "proptest-min", proptest(value = r#"Cord::new("math")"#))]
    #[cfg_attr(feature = "proptest-low", proptest(strategy = r#"r"[a-zA-Z0-9\s]{1,10}".prop_map(Cord::new)"#))]
    #[cfg_attr(feature = "proptest-high", proptest(strategy = r#"r"[^$]{1,100}".prop_map(Cord::new)"#))]
    #[cfg_attr(feature = "proptest-max", proptest(strategy = r#"String::arbitrary().prop_map(Cord::new)"#))]
    pub code: Cord,

    /// A digest of the `code` and `mathLanguage`.
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub compile_digest: Option<ExecutionDigest>,

    /// Errors that occurred when parsing the math equation.
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub errors: Option<Vec<String>>,

    /// The MathML transpiled from the `code`.
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    #[html(content)]
    pub mathml: Option<String>,
}

impl MathFragment {
    pub fn new(math_language: String, code: Cord) -> Self {
        Self {
            math_language,
            code,
            ..Default::default()
        }
    }
}
