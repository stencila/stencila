// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::organization::Organization;
use super::person::Person;
use super::software_application::SoftwareApplication;

/// [`Person`] or [`Organization`] or [`SoftwareApplication`]
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec, WriteNode, SmartDefault, ReadNode)]
#[serde(untagged, crate = "common::serde")]
pub enum PersonOrOrganizationOrSoftwareApplication {
    #[default]
    Person(Person),

    Organization(Organization),

    SoftwareApplication(SoftwareApplication),
}
