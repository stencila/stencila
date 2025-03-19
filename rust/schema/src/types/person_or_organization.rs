// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::organization::Organization;
use super::person::Person;

/// [`Person`] or [`Organization`]
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, ProbeNode, StripNode, WalkNode, WriteNode, SmartDefault, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, MarkdownCodec, TextCodec)]
#[serde(untagged, crate = "common::serde")]
pub enum PersonOrOrganization {
    #[default]
    Person(Person),

    Organization(Organization),
}
