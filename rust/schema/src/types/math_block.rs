// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::cord::Cord;
use super::execution_digest::ExecutionDigest;
use super::string::String;

/// A block of math, e.g an equation, to be treated as block content.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec, WriteNode, ReadNode)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[cfg_attr(feature = "proptest", derive(Arbitrary))]
#[derive(derive_more::Display)]
#[display(fmt = "MathBlock")]
#[html(elem = "math", custom)]
#[jats(elem = "disp-formula", special)]
#[markdown(special)]
pub struct MathBlock {
    /// The type of this item.
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    pub r#type: MustBe!("MathBlock"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The language used for the equation e.g tex, mathml, asciimath.
    #[serde(alias = "math-language", alias = "math_language")]
    #[cfg_attr(feature = "proptest-min", proptest(value = r#"String::from("lang")"#))]
    #[cfg_attr(feature = "proptest-low", proptest(regex = r#"(asciimath)|(mathml)|(tex)"#))]
    #[cfg_attr(feature = "proptest-high", proptest(regex = r#"[a-zA-Z0-9]{1,10}"#))]
    #[cfg_attr(feature = "proptest-max", proptest(strategy = r#"String::arbitrary()"#))]
    pub math_language: String,

    /// The code of the equation in the `mathLanguage`.
    #[cfg_attr(feature = "proptest-min", proptest(value = r#"Cord::new("math")"#))]
    #[cfg_attr(feature = "proptest-low", proptest(strategy = r#"r"[a-zA-Z0-9 \t]{1,10}".prop_map(Cord::new)"#))]
    #[cfg_attr(feature = "proptest-high", proptest(strategy = r#"r"[^\p{C}]{1,100}".prop_map(Cord::new)"#))]
    #[cfg_attr(feature = "proptest-max", proptest(strategy = r#"String::arbitrary().prop_map(Cord::new)"#))]
    pub code: Cord,

    /// A digest of the `code` and `mathLanguage`.
    #[serde(alias = "compile-digest", alias = "compile_digest")]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub compile_digest: Option<ExecutionDigest>,

    /// Errors that occurred when parsing the math equation.
    #[serde(alias = "error")]
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub errors: Option<Vec<String>>,

    /// The MathML transpiled from the `code`.
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    #[html(content)]
    pub mathml: Option<String>,

    /// A short label for the math block.
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub label: Option<String>,
}

impl MathBlock {
    pub fn new(math_language: String, code: Cord) -> Self {
        Self {
            math_language,
            code,
            ..Default::default()
        }
    }
}
