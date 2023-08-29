// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::organization::Organization;
use super::person::Person;

/// [`Person`] or [`Organization`]
#[derive(Debug, Clone, PartialEq, Display, Serialize, Deserialize, Strip, Read, Write, ToHtml, ToText)]
#[serde(untagged, crate = "common::serde")]
pub enum PersonOrOrganization {
    Person(Person),
    Organization(Organization),
}
