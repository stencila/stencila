// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::organization::Organization;
use super::person::Person;
use super::software_application::SoftwareApplication;

/// [`Person`] or [`Organization`] or [`SoftwareApplication`]
#[derive(Debug, Display, Clone, PartialEq, Serialize, Deserialize, StripNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec, ReadNode, WriteNode)]
#[serde(untagged, crate = "common::serde")]
pub enum PersonOrOrganizationOrSoftwareApplication {
    Person(Person),
    Organization(Organization),
    SoftwareApplication(SoftwareApplication),
}
