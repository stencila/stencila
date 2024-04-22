// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::organization::Organization;
use super::person::Person;
use super::software_application::SoftwareApplication;
use super::thing::Thing;

/// Union type for things that can be an author in `AuthorRole`.
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, SmartDefault, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
#[serde(untagged, crate = "common::serde")]
pub enum AuthorRoleAuthor {
    #[default]
    Person(Person),

    Organization(Organization),

    SoftwareApplication(SoftwareApplication),

    Thing(Thing),
}
