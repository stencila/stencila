// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::execution_status::ExecutionStatus;
use super::inline::Inline;
use super::person_or_organization_or_software_application::PersonOrOrganizationOrSoftwareApplication;
use super::string::String;

/// An instruction to edit some inline content.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec, WriteNode, ReadNode)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[cfg_attr(feature = "proptest", derive(Arbitrary))]
#[derive(derive_more::Display)]
#[display(fmt = "InstructInline")]
#[markdown(special)]
pub struct InstructInline {
    /// The type of this item.
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    pub r#type: MustBe!("InstructInline"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The text of the instruction.
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    pub text: String,

    /// The agent that executed the instruction.
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub agent: Option<PersonOrOrganizationOrSoftwareApplication>,

    /// Status of the execution of the instruction.
    #[serde(alias = "execution-status", alias = "execution_status")]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub execution_status: Option<ExecutionStatus>,

    /// The content to which the instruction applies.
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[walk]
    #[cfg_attr(feature = "proptest-min", proptest(value = r#"None"#))]
    #[cfg_attr(feature = "proptest-low", proptest(strategy = r#"option::of(vec_inlines_non_recursive(1))"#))]
    #[cfg_attr(feature = "proptest-high", proptest(strategy = r#"option::of(vec_inlines_non_recursive(2))"#))]
    #[cfg_attr(feature = "proptest-max", proptest(strategy = r#"option::of(vec_inlines_non_recursive(4))"#))]
    pub content: Option<Vec<Inline>>,
}

impl InstructInline {
    pub fn new(text: String) -> Self {
        Self {
            text,
            ..Default::default()
        }
    }
}
