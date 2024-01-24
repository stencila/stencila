// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::author_role::AuthorRole;
use super::organization::Organization;
use super::person::Person;
use super::software_application::SoftwareApplication;

/// Union type for things that can be an author of a `CreativeWork` or other type.
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, DomCodec, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec, WriteNode, SmartDefault, ReadNode)]
#[serde(untagged, crate = "common::serde")]
pub enum Author {
    #[default]
    Person(Person),

    Organization(Organization),

    SoftwareApplication(SoftwareApplication),

    AuthorRole(AuthorRole),
}
