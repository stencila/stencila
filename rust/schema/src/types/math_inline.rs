// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::author::Author;
use super::compilation_digest::CompilationDigest;
use super::compilation_message::CompilationMessage;
use super::cord::Cord;
use super::image_object::ImageObject;
use super::provenance_count::ProvenanceCount;
use super::string::String;

/// A fragment of math, e.g a variable name, to be treated as inline content.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, ProbeNode, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, HtmlCodec, JatsCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[cfg_attr(feature = "proptest", derive(Arbitrary))]
#[derive(derive_more::Display)]
#[display("MathInline")]
#[patch(authors_on = "self")]
#[html(elem = "math")]
#[jats(elem = "inline-formula", special)]
pub struct MathInline {
    /// The type of this item.
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    pub r#type: MustBe!("MathInline"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The code of the equation in the `mathLanguage`.
    #[strip(code)]
    #[patch(format = "md", format = "smd", format = "myst", format = "ipynb", format = "qmd", format = "latex")]
    #[cfg_attr(feature = "proptest-min", proptest(value = r#"Cord::from("math")"#))]
    #[cfg_attr(feature = "proptest-low", proptest(strategy = r#"r"[a-zA-Z0-9]{1,10}".prop_map(Cord::from)"#))]
    #[cfg_attr(feature = "proptest-high", proptest(strategy = r#"r"[^\p{C}]{1,100}".prop_map(Cord::from)"#))]
    #[cfg_attr(feature = "proptest-max", proptest(strategy = r#"String::arbitrary().prop_map(Cord::from)"#))]
    pub code: Cord,

    /// The language used for the equation e.g tex, mathml, asciimath.
    #[serde(alias = "math-language", alias = "math_language")]
    #[strip(code)]
    #[patch(format = "md", format = "smd", format = "myst", format = "ipynb", format = "qmd")]
    #[cfg_attr(feature = "proptest-min", proptest(value = r#"Some(String::from("tex"))"#))]
    #[cfg_attr(feature = "proptest-low", proptest(value = r#"Some(String::from("tex"))"#))]
    #[cfg_attr(feature = "proptest-high", proptest(strategy = r#"option::of(r"[a-zA-Z0-9]{1,10}")"#))]
    #[cfg_attr(feature = "proptest-max", proptest(strategy = r#"option::of(String::arbitrary())"#))]
    pub math_language: Option<String>,

    /// The authors of the math.
    #[serde(alias = "author")]
    #[serde(default, deserialize_with = "option_one_or_many_string_or_object")]
    #[strip(authors)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub authors: Option<Vec<Author>>,

    /// A summary of the provenance of the math.
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[strip(provenance)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub provenance: Option<Vec<ProvenanceCount>>,

    /// Non-core optional fields
    #[serde(flatten)]
    #[html(flatten)]
    #[jats(flatten)]
    pub options: Box<MathInlineOptions>,

    /// A unique identifier for a node within a document
    #[serde(skip)]
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    pub uid: NodeUid
}

#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, ProbeNode, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, HtmlCodec, JatsCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[cfg_attr(feature = "proptest", derive(Arbitrary))]
pub struct MathInlineOptions {
    /// A digest of the `code` and `mathLanguage`.
    #[serde(alias = "compilation-digest", alias = "compilation_digest")]
    #[strip(compilation)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub compilation_digest: Option<CompilationDigest>,

    /// Messages generated while parsing and compiling the math expression.
    #[serde(alias = "compilation-messages", alias = "compilation_messages", alias = "compilationMessage", alias = "compilation-message", alias = "compilation_message")]
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[strip(compilation)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub compilation_messages: Option<Vec<CompilationMessage>>,

    /// The MathML transpiled from the `code`.
    #[strip(output)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    #[html(content)]
    pub mathml: Option<String>,

    /// Images of the math.
    #[serde(alias = "image")]
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[strip(content)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub images: Option<Vec<ImageObject>>,
}

impl MathInline {
    const NICK: [u8; 3] = *b"mti";
    
    pub fn node_type(&self) -> NodeType {
        NodeType::MathInline
    }

    pub fn node_id(&self) -> NodeId {
        NodeId::new(&Self::NICK, &self.uid)
    }
    
    pub fn new(code: Cord) -> Self {
        Self {
            code,
            ..Default::default()
        }
    }
}
